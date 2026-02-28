<script lang="ts">
  /**
   * PropertyWorld — entity property inspection on a world canvas.
   *
   * Shows entities as large cards with their property values displayed
   * prominently. Groups entities by type. Syncs with runtime to reflect
   * live property changes. Double-click to jump to source.
   * Positions persist across recompilations.
   */

  import { onMount, onDestroy } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { projectionRegistry } from '$lib/app/projections/ProjectionRegistry';
  import { playbackService } from '$lib/app/views/runtime/_shared/PlaybackService.svelte';
  import { selectionContext } from '$lib/framework/selection/SelectionContext';
  import { navigationBroker } from '$lib/framework/navigation/NavigationBroker';
  import type {
    UrdWorld,
    PropertyDependencyIndex,
    PropertyDependencyEntry,
  } from '$lib/app/compiler/types';
  import type { WorldData, WorldLocation, WorldEntity, WorldRuntimeOverlay } from '../_shared/world-types';
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

  const userPositions = new Map<string, { x: number; y: number }>();
  const unsubscribers: (() => void)[] = [];

  onMount(() => {
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
  });

  onDestroy(() => {
    for (const unsub of unsubscribers) unsub();
  });

  function buildWorld(): void {
    const urdJson = projectionRegistry.get<UrdWorld>('urd.projection.urdJson');
    const propIndex = projectionRegistry.get<PropertyDependencyIndex>('urd.projection.propertyDependencyIndex');

    if (!urdJson || urdJson.entities.length === 0) {
      worldData = null;
      return;
    }

    const typeDefs = urdJson.types ?? {};

    // Build property dependency lookup
    const propDeps = new Map<string, PropertyDependencyEntry>();
    if (propIndex) {
      for (const entry of propIndex.properties) {
        propDeps.set(`${entry.entity_type}.${entry.property}`, entry);
      }
    }

    // Group entities by type
    const typeGroups = new Map<string, typeof urdJson.entities>();
    for (const entity of urdJson.entities) {
      const type = entity.type ?? '__untyped__';
      const group = typeGroups.get(type) ?? [];
      group.push(entity);
      typeGroups.set(type, group);
    }

    // Layout: one card per entity, grouped by type
    const worldLocations: WorldLocation[] = [];
    let groupY = 0;

    for (const [typeName, entities] of typeGroups) {
      const typeDef = typeDefs[typeName];
      const typeProps = typeDef?.properties ?? {};

      for (let i = 0; i < entities.length; i++) {
        const entity = entities[i];

        const propEntries: WorldEntity[] = Object.entries(entity.properties).map(([key, value]) => {
          const dep = propDeps.get(`${typeName}.${key}`);
          const propDef = typeProps[key];
          return {
            id: `${entity.id}.${key}`,
            name: `${key}: ${formatValue(value)}`,
            type: propDef?.type ?? typeof value,
            entityKind: 'property' as const,
            properties: {
              value,
              reads: dep?.read_count ?? 0,
              writes: dep?.write_count ?? 0,
              orphaned: dep?.orphaned ?? null,
              type: propDef?.type,
              default: propDef?.default,
            },
          };
        });

        const width = 260;
        const height = 80 + propEntries.length * 28;

        const saved = userPositions.get(entity.id);
        const x = saved ? saved.x : i * 320 - (entities.length * 320) / 2;
        const y = saved ? saved.y : groupY;

        worldLocations.push({
          id: entity.id,
          name: entity.name || entity.id,
          description: typeName !== '__untyped__' ? `Type: ${typeName}` : 'Untyped entity',
          x,
          y,
          width,
          height,
          isStart: false,
          entities: propEntries,
          exits: [],
        });
      }

      groupY += 300;
    }

    worldData = {
      locations: worldLocations,
      scenes: [],
      worldName: urdJson.world?.name ? `${urdJson.world.name} — Properties` : 'Property Inspector',
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
      currentLocationId: null,
      visitedLocationIds: new Set(),
      visitedExitKeys: new Set(),
      entityStates,
      turnCount: ps.turnCount,
      isPlaying: true,
    };
  }

  function handleLocationClick(entityId: string): void {
    selectionContext.select([{ kind: 'entity', id: entityId }]);
  }

  function handleLocationDblClick(entityId: string): void {
    navigationBroker.navigate({ targetViewId: 'urd.codeEditor', params: { searchSymbol: entityId } });
  }

  function handleEntityDblClick(entityPropId: string): void {
    const entityId = entityPropId.split('.')[0];
    navigationBroker.navigate({ targetViewId: 'urd.codeEditor', params: { searchSymbol: entityId } });
  }

  function handleNodeMoved(id: string, x: number, y: number): void {
    userPositions.set(id, { x, y });
    const posObj: Record<string, { x: number; y: number }> = {};
    for (const [k, v] of userPositions) posObj[k] = v;
    _onStateChange({ userPositions: posObj });
  }

  function formatValue(v: unknown): string {
    if (v === null || v === undefined) return 'null';
    if (typeof v === 'boolean') return v ? 'true' : 'false';
    if (typeof v === 'string') return `"${v}"`;
    return String(v);
  }
</script>

<WorldCanvas
  data={worldData ?? { locations: [], scenes: [], worldName: '' }}
  {runtimeOverlay}
  onLocationClick={handleLocationClick}
  onLocationDblClick={handleLocationDblClick}
  onEntityDblClick={handleEntityDblClick}
  onNodeMoved={handleNodeMoved}
  emptyMessage="No entity data to display"
/>
