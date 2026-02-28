<script lang="ts">
  import ForceGraphView from './ForceGraphView.svelte';
  import { selectionContext } from '$lib/framework/selection/SelectionContext';
  import { navigationBroker } from '$lib/framework/navigation/NavigationBroker';

  interface Props { zoneId: string; zoneTypeId: string; state: any; onStateChange: any; }
  let { zoneId, zoneTypeId, state, onStateChange }: Props = $props();

  function handleNodeClick(nodeId: string) {
    if (nodeId.startsWith('rule:')) {
      selectionContext.select([{ kind: 'rule', id: nodeId.slice(5) }]);
    } else if (nodeId.startsWith('prop:')) {
      selectionContext.select([{ kind: 'property', id: nodeId.slice(5) }]);
    }
  }

  function handleNodeDblClick(nodeId: string) {
    if (nodeId.startsWith('rule:')) {
      navigationBroker.navigate({ targetViewId: 'urd.codeEditor', params: { searchSymbol: nodeId.slice(5) } });
    }
  }
</script>

<ForceGraphView
  {zoneId} {zoneTypeId} {state} {onStateChange}
  projectionId="urd.projection.ruleTriggerNetwork"
  onNodeClick={handleNodeClick}
  onNodeDblClick={handleNodeDblClick}
  emptyMessage="No rule triggers to display"
/>
