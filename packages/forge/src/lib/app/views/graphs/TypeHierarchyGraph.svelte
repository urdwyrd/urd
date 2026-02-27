<script lang="ts">
  /**
   * Type Hierarchy Graph — type definitions with trait inheritance edges.
   * Consumes the typeHierarchy projection via GraphCanvas.
   */

  import { onMount, onDestroy } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { projectionRegistry } from '$lib/app/projections/ProjectionRegistry';
  import { selectionContext } from '$lib/framework/selection/SelectionContext';
  import type { ForgeGraphData } from './_shared/graph-types';
  import GraphCanvas from './_shared/GraphCanvas.svelte';

  interface Props {
    zoneId: string;
    zoneTypeId: string;
    state: { viewport: null };
    onStateChange: (newState: unknown) => void;
  }

  let { zoneId, zoneTypeId, state: zoneState, onStateChange }: Props = $props();

  let graphData: ForgeGraphData | null = $state(null);

  const unsubscribers: (() => void)[] = [];

  onMount(() => {
    refreshData();
    unsubscribers.push(bus.subscribe('compiler.completed', refreshData));
  });

  onDestroy(() => {
    for (const unsub of unsubscribers) unsub();
  });

  function refreshData(): void {
    graphData = projectionRegistry.get<ForgeGraphData>('urd.projection.typeHierarchy');
  }

  function handleNodeClick(nodeId: string): void {
    selectionContext.select([{ kind: 'type', id: nodeId }]);
  }
</script>

{#if graphData && graphData.nodes.length > 0}
  <GraphCanvas
    data={graphData}
    rankdir="BT"
    onNodeClick={handleNodeClick}
    emptyMessage="No types defined"
  />
{:else}
  <div class="forge-graph-view__empty">
    <p>No type hierarchy to display</p>
    <p class="forge-graph-view__hint">Type definitions appear after successful compilation</p>
  </div>
{/if}

<style>
  .forge-graph-view__empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    color: var(--forge-text-muted);
    font-size: var(--forge-font-size-sm);
    gap: var(--forge-space-sm);
  }

  .forge-graph-view__hint {
    font-size: var(--forge-font-size-xs);
    opacity: 0.6;
  }
</style>
