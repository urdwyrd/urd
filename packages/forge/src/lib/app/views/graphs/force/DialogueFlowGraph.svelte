<script lang="ts">
  import ForceGraphView from './ForceGraphView.svelte';
  import { selectionContext } from '$lib/framework/selection/SelectionContext';

  interface Props { zoneId: string; zoneTypeId: string; state: any; onStateChange: any; }
  let { zoneId, zoneTypeId, state, onStateChange }: Props = $props();
</script>

<ForceGraphView
  {zoneId} {zoneTypeId} {state} {onStateChange}
  projectionId="urd.projection.dialogueGraph"
  onNodeClick={(id) => { if (!id.startsWith('__')) selectionContext.select([{ kind: 'section', id }]); }}
  emptyMessage="No dialogue flow to display"
/>
