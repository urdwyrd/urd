<script lang="ts">
  import ForceGraphView from './ForceGraphView.svelte';
  import { selectionContext } from '$lib/framework/selection/SelectionContext';
  import { navigationBroker } from '$lib/framework/navigation/NavigationBroker';

  interface Props { zoneId: string; zoneTypeId: string; state: any; onStateChange: any; }
  let { zoneId, zoneTypeId, state, onStateChange }: Props = $props();

  function handleNodeClick(nodeId: string) {
    if (nodeId.startsWith('choice:')) {
      selectionContext.select([{ kind: 'choice', id: nodeId.slice(7) }]);
    } else if (!nodeId.startsWith('__')) {
      selectionContext.select([{ kind: 'section', id: nodeId }]);
    }
  }

  function handleNodeDblClick(nodeId: string) {
    if (nodeId.startsWith('choice:')) {
      navigationBroker.navigate({ targetViewId: 'urd.codeEditor', params: { searchSymbol: nodeId.slice(7) } });
    } else if (!nodeId.startsWith('__')) {
      navigationBroker.navigate({ targetViewId: 'urd.codeEditor', params: { searchSymbol: nodeId } });
    }
  }
</script>

<ForceGraphView
  {zoneId} {zoneTypeId} {state} {onStateChange}
  projectionId="urd.projection.choiceTree"
  onNodeClick={handleNodeClick}
  onNodeDblClick={handleNodeDblClick}
  emptyMessage="No choices to display"
/>
