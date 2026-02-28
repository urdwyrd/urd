<script lang="ts">
  import ForceGraphView from './ForceGraphView.svelte';
  import { selectionContext } from '$lib/framework/selection/SelectionContext';
  import type { PropertyDependencyIndex } from '$lib/app/compiler/types';
  import type { ForgeGraphData, ForgeGraphNode, ForgeGraphEdge } from '../_shared/graph-types';

  interface Props { zoneId: string; zoneTypeId: string; state: any; onStateChange: any; }
  let { zoneId, zoneTypeId, state, onStateChange }: Props = $props();

  function buildData(raw: unknown): ForgeGraphData {
    const index = raw as PropertyDependencyIndex;
    const nodes: ForgeGraphNode[] = [];
    const edges: ForgeGraphEdge[] = [];

    for (const prop of index.properties) {
      const nodeId = `${prop.entity_type}.${prop.property}`;
      nodes.push({
        id: nodeId,
        label: `${prop.entity_type}.${prop.property}`,
        kind: 'property' as const,
        flags: { orphaned: prop.orphaned !== null },
        metadata: { reads: prop.read_count, writes: prop.write_count, orphaned: prop.orphaned },
      });
    }

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
</script>

<ForceGraphView
  {zoneId} {zoneTypeId} {state} {onStateChange}
  projectionId="urd.projection.propertyDependencyIndex"
  {buildData}
  onNodeClick={(id) => selectionContext.select([{ kind: 'property', id }])}
  emptyMessage="No property data flow to display"
/>
