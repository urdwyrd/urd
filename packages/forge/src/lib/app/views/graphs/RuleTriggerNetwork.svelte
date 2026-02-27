<script lang="ts">
  /**
   * Rule Trigger Network — rules and properties as nodes, condition/effect links as edges.
   * Consumes the ruleTriggerNetwork projection via GraphCanvas.
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
    graphData = projectionRegistry.get<ForgeGraphData>('urd.projection.ruleTriggerNetwork');
  }

  function handleNodeClick(nodeId: string): void {
    if (nodeId.startsWith('rule:')) {
      selectionContext.select([{ kind: 'rule', id: nodeId.slice(5) }]);
    } else if (nodeId.startsWith('prop:')) {
      selectionContext.select([{ kind: 'property', id: nodeId.slice(5) }]);
    }
  }

  function handleNodeDblClick(nodeId: string): void {
    if (nodeId.startsWith('rule:')) {
      navigationBroker.navigate({
        targetViewId: 'urd.codeEditor',
        params: { searchSymbol: nodeId.slice(5) },
      });
    }
  }
</script>

{#if graphData && graphData.nodes.length > 0}
  <GraphCanvas
    data={graphData}
    rankdir="LR"
    onNodeClick={handleNodeClick}
    onNodeDblClick={handleNodeDblClick}
    emptyMessage="No rules found"
  />
{:else}
  <div class="forge-graph-view__empty">
    <p>No rule triggers to display</p>
    <p class="forge-graph-view__hint">Rule triggers appear when rules read or write properties</p>
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
