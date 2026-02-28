<script lang="ts">
  /**
   * DialogueWorld — a spatial dialogue flow view on the world canvas.
   *
   * Shows sections as cards with choices inside, jump edges connecting
   * sections. Double-click to jump to source. Positions persist across
   * recompilations.
   */

  import { onMount, onDestroy } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { projectionRegistry } from '$lib/app/projections/ProjectionRegistry';
  import { playbackService } from '$lib/app/views/runtime/_shared/PlaybackService.svelte';
  import { selectionContext } from '$lib/framework/selection/SelectionContext';
  import { navigationBroker } from '$lib/framework/navigation/NavigationBroker';
  import type { UrdWorld, FactSet } from '$lib/app/compiler/types';
  import type { WorldData, WorldLocation, WorldRuntimeOverlay } from '../_shared/world-types';
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
    const factSet = projectionRegistry.get<FactSet>('urd.projection.factSet');

    if (!urdJson || !factSet) {
      worldData = null;
      return;
    }

    const sectionIds = new Set<string>();
    for (const c of factSet.choices) {
      sectionIds.add(c.section);
    }

    // Build jump graph
    const sectionJumps = new Map<string, string[]>();
    for (const jump of factSet.jumps) {
      if (jump.target.kind === 'section' && jump.target.id) {
        const targets = sectionJumps.get(jump.from_section) ?? [];
        targets.push(jump.target.id);
        sectionJumps.set(jump.from_section, targets);
      }
    }

    const sectionList = [...sectionIds];
    const cols = Math.max(1, Math.ceil(Math.sqrt(sectionList.length)));

    const worldLocations: WorldLocation[] = sectionList.map((sectionId, i) => {
      const col = i % cols;
      const row = Math.floor(i / cols);

      const sectionChoices = factSet.choices.filter((c) => c.section === sectionId);

      const width = 240;
      const height = 80 + sectionChoices.length * 28;

      const saved = userPositions.get(sectionId);
      const x = saved ? saved.x : col * 340 - (cols * 340) / 2;
      const y = saved ? saved.y : row * 260 - (Math.ceil(sectionList.length / cols) * 260) / 2;

      return {
        id: sectionId,
        name: sectionId,
        description: `${sectionChoices.length} choice${sectionChoices.length !== 1 ? 's' : ''}`,
        x,
        y,
        width,
        height,
        isStart: i === 0,
        entities: sectionChoices.map((c) => ({
          id: c.choice_id,
          name: c.label,
          type: c.sticky ? 'sticky' : 'oneshot',
          entityKind: c.sticky ? 'item' as const : 'scenery' as const,
          properties: {
            sticky: c.sticky,
            conditions: c.condition_reads.length,
            effects: c.effect_writes.length,
          },
        })),
        exits: (sectionJumps.get(sectionId) ?? []).map((targetId) => ({
          direction: 'jump',
          targetId,
          isConditional: false,
        })),
      };
    });

    worldData = {
      locations: worldLocations,
      scenes: [],
      worldName: urdJson.world?.name ? `${urdJson.world.name} — Dialogue` : 'Dialogue Flow',
    };
  }

  function buildRuntimeOverlay(): void {
    const ps = playbackService.state;
    if (ps.status !== 'playing') {
      runtimeOverlay = null;
      return;
    }

    runtimeOverlay = {
      currentLocationId: ps.activeSection,
      visitedLocationIds: new Set(playbackService.coverage.sections.visited),
      visitedExitKeys: new Set(),
      entityStates: new Map(),
      turnCount: ps.turnCount,
      isPlaying: true,
    };
  }

  function handleLocationClick(sectionId: string): void {
    selectionContext.select([{ kind: 'section', id: sectionId }]);
  }

  function handleLocationDblClick(sectionId: string): void {
    navigationBroker.navigate({ targetViewId: 'urd.codeEditor', params: { searchSymbol: sectionId } });
  }

  function handleEntityDblClick(choiceId: string): void {
    navigationBroker.navigate({ targetViewId: 'urd.codeEditor', params: { searchSymbol: choiceId } });
  }

  function handleNodeMoved(id: string, x: number, y: number): void {
    userPositions.set(id, { x, y });
    const posObj: Record<string, { x: number; y: number }> = {};
    for (const [k, v] of userPositions) posObj[k] = v;
    _onStateChange({ userPositions: posObj });
  }
</script>

<WorldCanvas
  data={worldData ?? { locations: [], scenes: [], worldName: '' }}
  {runtimeOverlay}
  onLocationClick={handleLocationClick}
  onLocationDblClick={handleLocationDblClick}
  onEntityDblClick={handleEntityDblClick}
  onNodeMoved={handleNodeMoved}
  emptyMessage="No dialogue data to display"
/>
