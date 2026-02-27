<script lang="ts">
  /**
   * Choice Tree — sections and their choices as a tree structure.
   * Consumes the choiceTree projection via GraphCanvas.
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
    graphData = projectionRegistry.get<ForgeGraphData>('urd.projection.choiceTree');
  }

  function handleNodeClick(nodeId: string): void {
    if (nodeId.startsWith('choice:')) {
      selectionContext.select([{ kind: 'choice', id: nodeId.slice(7) }]);
    } else if (!nodeId.startsWith('__')) {
      selectionContext.select([{ kind: 'section', id: nodeId }]);
    }
  }

  function handleNodeDblClick(nodeId: string): void {
    if (nodeId.startsWith('choice:')) {
      navigationBroker.navigate({
        targetViewId: 'urd.codeEditor',
        params: { searchSymbol: nodeId.slice(7) },
      });
    } else if (!nodeId.startsWith('__')) {
      navigationBroker.navigate({
        targetViewId: 'urd.codeEditor',
        params: { searchSymbol: nodeId },
      });
    }
  }
</script>

{#if graphData && graphData.nodes.length > 0}
  <GraphCanvas
    data={graphData}
    rankdir="TB"
    onNodeClick={handleNodeClick}
    onNodeDblClick={handleNodeDblClick}
    emptyMessage="No choices found"
  />
{:else}
  <div class="forge-graph-view__empty">
    <p>No choice tree to display</p>
    <p class="forge-graph-view__hint">Choices appear when sections contain choice blocks</p>
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
