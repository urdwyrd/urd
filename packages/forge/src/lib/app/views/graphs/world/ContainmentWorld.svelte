<script lang="ts">
  /**
   * ContainmentWorld — the primary Graph World view.
   *
   * Shows the full Urd world as a spatial map: locations as large cards,
   * entities inside them with type-based icons, exit arrows with condition
   * labels, scenes with choice trees, and a live runtime overlay.
   *
   * Manually placed positions persist across recompilations.
   * Double-click jumps to the source definition in the code editor.
   */

  import { onMount, onDestroy } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { projectionRegistry } from '$lib/app/projections/ProjectionRegistry';
  import { playbackService } from '$lib/app/views/runtime/_shared/PlaybackService.svelte';
  import { selectionContext } from '$lib/framework/selection/SelectionContext';
  import { navigationBroker } from '$lib/framework/navigation/NavigationBroker';
  import type { UrdWorld, UrdLocation, FactSet, PropertyRead } from '$lib/app/compiler/types';
  import type { WorldData, WorldLocation, WorldEntity, WorldExit, WorldScene, WorldRuntimeOverlay } from '../_shared/world-types';
  import { resolveEntityKind } from '../_shared/world-types';
  import WorldCanvas from '../_shared/WorldCanvas.svelte';

  interface Props {
    zoneId: string;
    zoneTypeId: string;
    state: unknown;
    onStateChange: (newState: unknown) => void;
  }

  let { zoneId, zoneTypeId, state: _zoneState, onStateChange: _onStateChange }: Props = $props();

  let worldData: WorldData | null = $state(null);
  let runtimeOverlay: WorldRuntimeOverlay | null = $state(null);

  // User-placed positions — survive recompilation
  const userPositions = new Map<string, { x: number; y: number }>();

  const unsubscribers: (() => void)[] = [];

  // Layout constants
  const CELL_SIZE = 300;
  const LOC_MIN_W = 220;
  const LOC_HEADER_H = 40;
  const LOC_DESC_H = 24;
  const ENTITY_ROW_H = 28;
  const ENTITY_COL_W = 150;
  const LOC_PADDING = 16;

  const DIRECTION_OFFSETS: Record<string, [number, number]> = {
    north: [0, -1], south: [0, 1], east: [1, 0], west: [-1, 0],
    northeast: [1, -1], northwest: [-1, -1], southeast: [1, 1], southwest: [-1, 1],
    up: [0, -1], down: [0, 1],
  };

  onMount(() => {
    // Restore saved positions from zone state
    const saved = _zoneState as { userPositions?: Record<string, { x: number; y: number }> } | null;
    if (saved?.userPositions) {
      for (const [id, pos] of Object.entries(saved.userPositions)) {
        userPositions.set(id, pos);
      }
    }

    buildWorld();
    buildRuntimeOverlay();
    unsubscribers.push(bus.subscribe('compiler.completed', buildWorld));
    unsubscribers.push(bus.subscribe('playback.state.changed', buildRuntimeOverlay));
    unsubscribers.push(bus.subscribe('coverage.overlay.updated', buildRuntimeOverlay));
  });

  onDestroy(() => {
    for (const unsub of unsubscribers) unsub();
  });

  function buildWorld(): void {
    const urdJson = projectionRegistry.get<UrdWorld>('urd.projection.urdJson');
    const factSet = projectionRegistry.get<FactSet>('urd.projection.factSet');

    if (!urdJson || urdJson.locations.length === 0) {
      worldData = null;
      return;
    }

    // Build location map
    const locMap = new Map<string, UrdLocation>();
    for (const loc of urdJson.locations) {
      locMap.set(loc.id, loc);
    }

    // Build exit conditionality index from factSet
    const exitConditions = new Map<string, string>();
    if (factSet) {
      for (const edge of factSet.exits) {
        const key = `${edge.from_location}/${edge.exit_name}`;
        if (edge.is_conditional && edge.guard_reads.length > 0) {
          const conditions = edge.guard_reads
            .map((ri) => factSet.reads[ri])
            .filter(Boolean)
            .map((r: PropertyRead) => `${r.entity_type}.${r.property} ${r.operator} ${r.value_literal}`)
            .join(' & ');
          exitConditions.set(key, conditions);
        }
      }
    }

    // Build conditional exit set
    const conditionalExits = new Set<string>();
    if (factSet) {
      for (const edge of factSet.exits) {
        if (edge.is_conditional) {
          conditionalExits.add(`${edge.from_location}/${edge.exit_name}`);
        }
      }
    }

    // Type definitions for entity kind resolution
    const typeDefs = urdJson.types ?? {};

    // BFS grid layout from start location
    const startId = urdJson.world?.start ?? urdJson.locations[0]?.id;
    const positions = new Map<string, [number, number]>();

    if (startId) {
      positions.set(startId, [0, 0]);
      const queue: string[] = [startId];
      const visited = new Set<string>([startId]);

      while (queue.length > 0) {
        const currentId = queue.shift()!;
        const loc = locMap.get(currentId);
        if (!loc) continue;

        const [cx, cy] = positions.get(currentId)!;

        for (const exit of loc.exits) {
          if (visited.has(exit.target)) continue;
          const offset = DIRECTION_OFFSETS[exit.direction.toLowerCase()] ?? [1, 0];
          let nx = cx + offset[0];
          let ny = cy + offset[1];

          // Avoid collisions
          let attempts = 0;
          while ([...positions.values()].some(([px, py]) => px === nx && py === ny) && attempts < 12) {
            nx += offset[0] || 1;
            ny += offset[1] || 1;
            attempts++;
          }

          positions.set(exit.target, [nx, ny]);
          visited.add(exit.target);
          queue.push(exit.target);
        }
      }

      // Place unvisited locations
      let unvisitedCol = 0;
      const maxY = Math.max(0, ...[...positions.values()].map(([, y]) => y));
      for (const loc of urdJson.locations) {
        if (!positions.has(loc.id)) {
          positions.set(loc.id, [unvisitedCol, maxY + 2]);
          unvisitedCol++;
        }
      }
    }

    // Build entity map (entity ID → entity)
    const entityMap = new Map(urdJson.entities.map((e) => [e.id, e]));

    // Build WorldLocation objects
    const worldLocations: WorldLocation[] = urdJson.locations.map((loc) => {
      const [gx, gy] = positions.get(loc.id) ?? [0, 0];
      const containedIds = new Set(loc.contains ?? []);

      // Resolve entities for this location
      const entities: WorldEntity[] = [];
      if (containedIds.size > 0) {
        for (const eid of containedIds) {
          const entity = entityMap.get(eid);
          if (entity) {
            const typeDef = typeDefs[entity.type ?? ''];
            entities.push({
              id: entity.id,
              name: entity.name || entity.id,
              type: entity.type,
              entityKind: resolveEntityKind(entity.type, typeDef?.traits),
              properties: { ...entity.properties },
            });
          }
        }
      }

      // Resolve exits
      const exits: WorldExit[] = loc.exits.map((exit) => {
        const key = `${loc.id}/${exit.direction}`;
        return {
          direction: exit.direction,
          targetId: exit.target,
          isConditional: conditionalExits.has(key),
          conditionLabel: exitConditions.get(key),
        };
      });

      // Compute card dimensions
      const entityCols = Math.max(1, Math.min(2, entities.length));
      const entityRows = Math.ceil(entities.length / entityCols);
      const entitiesH = entities.length > 0 ? entityRows * ENTITY_ROW_H + 12 : 0;
      const width = Math.max(LOC_MIN_W, entityCols * ENTITY_COL_W + LOC_PADDING * 2);
      const height = LOC_HEADER_H + LOC_DESC_H + entitiesH + LOC_PADDING;

      // Use saved position if available, otherwise BFS-computed position
      const saved = userPositions.get(loc.id);
      const x = saved ? saved.x : gx * CELL_SIZE - width / 2;
      const y = saved ? saved.y : gy * CELL_SIZE - height / 2;

      return {
        id: loc.id,
        name: loc.name || loc.id,
        description: loc.description || '',
        x,
        y,
        width,
        height,
        isStart: loc.id === startId,
        entities,
        exits,
      };
    });

    // Build scenes from factSet choices
    const scenes: WorldScene[] = [];
    if (factSet) {
      const sectionMap = new Map<string, WorldScene>();
      for (const choice of factSet.choices) {
        let scene = sectionMap.get(choice.section);
        if (!scene) {
          scene = {
            id: choice.section,
            choices: [],
            x: 0,
            y: 0,
          };
          sectionMap.set(choice.section, scene);
        }
        scene.choices.push({
          id: choice.choice_id,
          label: choice.label,
          sticky: choice.sticky,
          effectCount: choice.effect_writes.length,
        });
      }

      // Position scenes offset from locations
      let sceneOffset = 0;
      for (const scene of sectionMap.values()) {
        const maxX = Math.max(0, ...worldLocations.map((l) => l.x + l.width));
        scene.x = maxX + 120 + (sceneOffset % 3) * 200;
        scene.y = Math.floor(sceneOffset / 3) * 160 - (sectionMap.size * 160) / 6;
        sceneOffset++;
        scenes.push(scene);
      }
    }

    worldData = {
      locations: worldLocations,
      scenes,
      worldName: urdJson.world?.name ?? '',
    };
  }

  function buildRuntimeOverlay(): void {
    const ps = playbackService.state;
    if (ps.status !== 'playing') {
      runtimeOverlay = null;
      return;
    }

    const entityStates = new Map<string, Record<string, unknown>>();
    for (const entity of ps.entities) {
      entityStates.set(entity.id, { ...entity.properties });
    }

    runtimeOverlay = {
      currentLocationId: ps.currentLocation?.id ?? null,
      visitedLocationIds: new Set(playbackService.coverage.locations.visited),
      visitedExitKeys: new Set(playbackService.coverage.exits.visited),
      entityStates,
      turnCount: ps.turnCount,
      isPlaying: true,
    };
  }

  function handleLocationClick(locationId: string): void {
    selectionContext.select([{ kind: 'location', id: locationId }]);
  }

  function handleEntityClick(entityId: string): void {
    selectionContext.select([{ kind: 'entity', id: entityId }]);
  }

  function handleLocationDblClick(locationId: string): void {
    navigationBroker.navigate({ targetViewId: 'urd.codeEditor', params: { searchSymbol: locationId } });
  }

  function handleEntityDblClick(entityId: string): void {
    navigationBroker.navigate({ targetViewId: 'urd.codeEditor', params: { searchSymbol: entityId } });
  }

  function handleNodeMoved(locationId: string, x: number, y: number): void {
    userPositions.set(locationId, { x, y });
    // Persist to zone state
    const posObj: Record<string, { x: number; y: number }> = {};
    for (const [id, pos] of userPositions) {
      posObj[id] = pos;
    }
    _onStateChange({ userPositions: posObj });
  }
</script>

<WorldCanvas
  data={worldData ?? { locations: [], scenes: [], worldName: '' }}
  {runtimeOverlay}
  onLocationClick={handleLocationClick}
  onEntityClick={handleEntityClick}
  onLocationDblClick={handleLocationDblClick}
  onEntityDblClick={handleEntityDblClick}
  onNodeMoved={handleNodeMoved}
  emptyMessage="No world data to display"
/>
