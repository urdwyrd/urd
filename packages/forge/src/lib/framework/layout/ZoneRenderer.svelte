<script lang="ts">
  /**
   * ZoneRenderer — recursive component that renders the BSP tree.
   * Renders SplitContainer for splits and ZoneShell for leaves.
   */

  import type { ZoneTree, ZoneTreeAction } from '../types';
  import { isLeaf } from './ZoneTree';
  import SplitContainer from './SplitContainer.svelte';
  import Divider from './Divider.svelte';
  import ZoneShell from './ZoneShell.svelte';
  import ZoneRenderer from './ZoneRenderer.svelte';
  import type { ZoneStateStore } from './ZoneStateStore';
  import { viewRegistry } from '../views/ViewRegistry';

  interface Props {
    node: ZoneTree;
    zoneStates: ZoneStateStore;
    projectOpen: boolean;
    dispatch: (action: ZoneTreeAction) => void;
    onZoneContextMenu: (e: MouseEvent, zoneId: string, zoneTypeId: string) => void;
    onDividerContextMenu: (e: MouseEvent, dividerId: string, direction: 'horizontal' | 'vertical') => void;
  }

  let { node, zoneStates, projectOpen, dispatch, onZoneContextMenu, onDividerContextMenu }: Props = $props();
</script>

{#if isLeaf(node)}
  {@const isSingleton = viewRegistry.isSingleton(node.zoneTypeId)}
  <ZoneShell
    zoneId={node.id}
    zoneTypeId={node.zoneTypeId}
    zoneState={isSingleton
      ? zoneStates.getSingletonState(node.zoneTypeId)
      : zoneStates.get(node.id, node.zoneTypeId)}
    {projectOpen}
    onChangeType={(typeId) => dispatch({ type: 'changeType', zoneId: node.id, newTypeId: typeId })}
    onSplit={(direction) => dispatch({ type: 'split', zoneId: node.id, direction })}
    onStateChange={(state) => isSingleton
      ? zoneStates.setSingletonState(node.zoneTypeId, state)
      : zoneStates.set(node.id, node.zoneTypeId, state)}
    onContextMenu={(e) => onZoneContextMenu(e, node.id, node.zoneTypeId)}
  />
{:else}
  <SplitContainer splitNode={node}>
    {#snippet first()}
      <ZoneRenderer
        node={node.children[0]}
        {zoneStates}
        {projectOpen}
        {dispatch}
        {onZoneContextMenu}
        {onDividerContextMenu}
      />
    {/snippet}
    {#snippet second()}
      <ZoneRenderer
        node={node.children[1]}
        {zoneStates}
        {projectOpen}
        {dispatch}
        {onZoneContextMenu}
        {onDividerContextMenu}
      />
    {/snippet}
    {#snippet divider()}
      <Divider
        splitNode={node}
        onResize={(ratio) => dispatch({ type: 'resize', dividerId: node.id, ratio })}
        onJoin={(keep) => dispatch({ type: 'join', dividerId: node.id, keep })}
        onSwap={() => dispatch({ type: 'swap', dividerId: node.id })}
        onReset={() => dispatch({ type: 'resetDivider', dividerId: node.id })}
        onContextMenu={(e) => onDividerContextMenu(e, node.id, node.direction)}
      />
    {/snippet}
  </SplitContainer>
{/if}
