<script lang="ts">
  /**
   * Property Data Flow Graph — properties as nodes, read/write relationships as edges.
   * Builds ForgeGraphData inline from propertyDependencyIndex projection.
   */

  import { onMount, onDestroy } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { projectionRegistry } from '$lib/app/projections/ProjectionRegistry';
  import { selectionContext } from '$lib/framework/selection/SelectionContext';
  import type { PropertyDependencyIndex } from '$lib/app/compiler/types';
  import type { ForgeGraphData, ForgeGraphNode, ForgeGraphEdge } from './_shared/graph-types';
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
    const index = projectionRegistry.get<PropertyDependencyIndex>(
      'urd.projection.propertyDependencyIndex',
    );
    if (!index) {
      graphData = null;
      return;
    }
    graphData = buildGraphFromIndex(index);
  }

  function buildGraphFromIndex(index: PropertyDependencyIndex): ForgeGraphData {
    const nodes: ForgeGraphNode[] = [];
    const edges: ForgeGraphEdge[] = [];

    for (const prop of index.properties) {
      const nodeId = `${prop.entity_type}.${prop.property}`;
      nodes.push({
        id: nodeId,
        label: `${prop.entity_type}.${prop.property}`,
        kind: 'property' as const,
        flags: {
          orphaned: prop.orphaned !== null,
        },
        metadata: {
          reads: prop.read_count,
          writes: prop.write_count,
          orphaned: prop.orphaned,
        },
      });
    }

    // Create edges between properties that share read/write indices
    // Properties that are written and then read by the same site suggest data flow
    const writeIndices = new Map<number, string>();
    for (const prop of index.properties) {
      const nodeId = `${prop.entity_type}.${prop.property}`;
      for (const wi of prop.write_indices) {
        writeIndices.set(wi, nodeId);
      }
    }

    for (const prop of index.properties) {
      const nodeId = `${prop.entity_type}.${prop.property}`;
      for (const ri of prop.read_indices) {
        const writer = writeIndices.get(ri);
        if (writer && writer !== nodeId) {
          edges.push({
            id: `flow:${writer}→${nodeId}:${ri}`,
            source: writer,
            target: nodeId,
            kind: 'reference',
          });
        }
      }
    }

    return { nodes, edges };
  }

  function handleNodeClick(nodeId: string): void {
    selectionContext.select([{ kind: 'property', id: nodeId }]);
  }
</script>

{#if graphData && graphData.nodes.length > 0}
  <GraphCanvas
    data={graphData}
    rankdir="LR"
    nodeWidth={180}
    onNodeClick={handleNodeClick}
    emptyMessage="No properties defined"
  />
{:else}
  <div class="forge-graph-view__empty">
    <p>No property data flow to display</p>
    <p class="forge-graph-view__hint">Properties appear after successful compilation</p>
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
