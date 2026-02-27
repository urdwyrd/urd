<script lang="ts">
  /**
   * File Dependency Graph — files as nodes, cross-file references as edges.
   * Consumes the fileDependencyGraph projection via GraphCanvas.
   */

  import { onMount, onDestroy } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { projectionRegistry } from '$lib/app/projections/ProjectionRegistry';
  import { selectionContext } from '$lib/framework/selection/SelectionContext';
  import { navigationBroker } from '$lib/framework/navigation/NavigationBroker';
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
    graphData = projectionRegistry.get<ForgeGraphData>('urd.projection.fileDependencyGraph');
  }

  function handleNodeClick(nodeId: string): void {
    selectionContext.select([{ kind: 'file', id: nodeId }]);
  }

  function handleNodeDblClick(nodeId: string): void {
    navigationBroker.navigate({
      targetViewId: 'urd.codeEditor',
      params: { file: nodeId },
    });
  }
</script>

{#if graphData && graphData.nodes.length > 0}
  <GraphCanvas
    data={graphData}
    rankdir="TB"
    onNodeClick={handleNodeClick}
    onNodeDblClick={handleNodeDblClick}
    emptyMessage="No file dependencies found"
  />
{:else}
  <div class="forge-graph-view__empty">
    <p>No file dependencies to display</p>
    <p class="forge-graph-view__hint">File dependencies appear when multiple files reference each other</p>
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
