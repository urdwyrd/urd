<script lang="ts">
  /**
   * Cross-Reference Graph — symbols and their references.
   * Consumes the crossReference projection via GraphCanvas.
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
    graphData = projectionRegistry.get<ForgeGraphData>('urd.projection.crossReference');
  }

  function handleNodeClick(nodeId: string): void {
    if (nodeId.startsWith('loc:')) {
      selectionContext.select([{ kind: 'location', id: nodeId.slice(4) }]);
    } else if (nodeId.startsWith('type:')) {
      selectionContext.select([{ kind: 'type', id: nodeId.slice(5) }]);
    }
  }
</script>

{#if graphData && graphData.nodes.length > 0}
  <GraphCanvas
    data={graphData}
    rankdir="LR"
    nodesep={40}
    onNodeClick={handleNodeClick}
    emptyMessage="No cross-references found"
  />
{:else}
  <div class="forge-graph-view__empty">
    <p>No cross-references to display</p>
    <p class="forge-graph-view__hint">References appear after successful compilation</p>
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
