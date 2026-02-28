<script lang="ts">
  import ForceGraphView from './ForceGraphView.svelte';
  import { selectionContext } from '$lib/framework/selection/SelectionContext';

  interface Props { zoneId: string; zoneTypeId: string; state: any; onStateChange: any; }
  let { zoneId, zoneTypeId, state, onStateChange }: Props = $props();

  function handleNodeClick(nodeId: string) {
    if (nodeId.startsWith('loc:')) {
      selectionContext.select([{ kind: 'location', id: nodeId.slice(4) }]);
    } else if (nodeId.startsWith('type:')) {
      selectionContext.select([{ kind: 'type', id: nodeId.slice(5) }]);
    }
  }
</script>

<ForceGraphView
  {zoneId} {zoneTypeId} {state} {onStateChange}
  projectionId="urd.projection.crossReference"
  onNodeClick={handleNodeClick}
  emptyMessage="No cross-references found"
/>
