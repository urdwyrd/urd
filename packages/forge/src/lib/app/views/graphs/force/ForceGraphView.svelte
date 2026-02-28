<script lang="ts">
  /**
   * Shared data-fetching wrapper for force-directed graph views.
   * Subscribes to compiler output, fetches projection, passes to ForceCanvas.
   */

  import { onMount, onDestroy } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { projectionRegistry } from '$lib/app/projections/ProjectionRegistry';
  import type { ForgeGraphData } from '../_shared/graph-types';
  import ForceCanvas from '../_shared/ForceCanvas.svelte';

  interface Props {
    zoneId: string;
    zoneTypeId: string;
    state?: unknown;
    onStateChange?: (newState: unknown) => void;
    projectionId: string;
    buildData?: (raw: unknown) => ForgeGraphData;
    onNodeClick?: (nodeId: string) => void;
    onNodeDblClick?: (nodeId: string) => void;
    emptyMessage?: string;
  }

  let {
    zoneId,
    zoneTypeId,
    state: _zoneState,
    onStateChange: _onStateChange,
    projectionId,
    buildData,
    onNodeClick,
    onNodeDblClick,
    emptyMessage = 'No data to display',
  }: Props = $props();

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
    const raw = projectionRegistry.get(projectionId);
    if (!raw) {
      graphData = null;
      return;
    }
    graphData = buildData ? buildData(raw) : (raw as ForgeGraphData);
  }
</script>

{#if graphData && graphData.nodes.length > 0}
  <ForceCanvas
    data={graphData}
    {onNodeClick}
    {onNodeDblClick}
    {emptyMessage}
  />
{:else}
  <div class="forge-graph-view__empty">
    <p>{emptyMessage}</p>
    <p class="forge-graph-view__hint">Data appears after successful compilation</p>
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
