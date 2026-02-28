<script lang="ts">
  import ForceGraphView from './ForceGraphView.svelte';
  import { selectionContext } from '$lib/framework/selection/SelectionContext';
  import { navigationBroker } from '$lib/framework/navigation/NavigationBroker';

  interface Props { zoneId: string; zoneTypeId: string; state: any; onStateChange: any; }
  let { zoneId, zoneTypeId, state, onStateChange }: Props = $props();
</script>

<ForceGraphView
  {zoneId} {zoneTypeId} {state} {onStateChange}
  projectionId="urd.projection.locationGraph"
  onNodeClick={(id) => selectionContext.select([{ kind: 'location', id }])}
  onNodeDblClick={(id) => navigationBroker.navigate({ targetViewId: 'urd.codeEditor', params: { searchSymbol: id } })}
  emptyMessage="No locations to display"
/>
