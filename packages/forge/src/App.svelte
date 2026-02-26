<script lang="ts">
  /**
   * Urd Forge — root application component.
   *
   * Component hierarchy from the architecture doc §5.5:
   * <App>
   *   <GlobalMenuBar />
   *   <WorkspaceTabs />
   *   <Workspace> or <WelcomeScreen>
   *   <GlobalStatusBar />
   * </App>
   */

  import { onMount } from 'svelte';
  import GlobalMenuBar from '$lib/framework/menu/GlobalMenuBar.svelte';
  import WorkspaceTabs from '$lib/framework/workspace/WorkspaceTabs.svelte';
  import ZoneRenderer from '$lib/framework/layout/ZoneRenderer.svelte';
  import GlobalStatusBar from '$lib/framework/layout/GlobalStatusBar.svelte';
  import WelcomeScreen from '$lib/framework/project/WelcomeScreen.svelte';
  import ContextMenu from '$lib/framework/context-menu/ContextMenu.svelte';
  import { workspaceManager } from '$lib/framework/workspace/WorkspaceManager.svelte';
  import { projectManager } from '$lib/framework/project/ProjectManager.svelte';
  import { bootstrap } from '$lib/app/bootstrap';
  import type { ContextMenuItem, SplitNode, ZoneTree as ZoneTreeType, ZoneTreeAction } from '$lib/framework/types';

  let ready = $state(false);
  let cleanup: (() => void) | null = null;

  // Context menu state
  let contextMenu = $state<{
    items: ContextMenuItem[];
    x: number;
    y: number;
  } | null>(null);

  // Split-positioning mode: after a split, the divider follows the mouse until click confirms
  let splitPositioning = $state<{
    dividerId: string;
    direction: 'horizontal' | 'vertical';
    zoneId: string; // original zone that was split, for undo
  } | null>(null);

  onMount(async () => {
    cleanup = await bootstrap();
    ready = true;

    // Show the window now that the app is rendered (hidden by default to prevent flash)
    if ('__TAURI_INTERNALS__' in window) {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      await getCurrentWindow().show();
    }

    return () => {
      cleanup?.();
    };
  });

  function dispatch(action: ZoneTreeAction) {
    // Intercept split actions to enter positioning mode
    if (action.type === 'split') {
      performSplit(action.zoneId, action.direction);
      return;
    }
    workspaceManager.dispatch(action);
  }

  /**
   * Find the DOM element for a split container by its data-split-id.
   */
  function findSplitElement(dividerId: string): HTMLElement | null {
    return document.querySelector(`[data-split-id="${dividerId}"]`);
  }

  /**
   * Enter split-positioning mode: split the zone, then let the user
   * position the divider by moving the mouse. Click to confirm, Escape to cancel.
   */
  function performSplit(zoneId: string, direction: 'horizontal' | 'vertical') {
    // Apply the split directly (bypass dispatch to avoid re-entry)
    workspaceManager.dispatch({ type: 'split', zoneId, direction });

    // The zone was replaced by a split node whose first child has the original zone's ID.
    // Walk the tree to find the split that contains it.
    const tree = workspaceManager.active.tree;
    const parentSplit = findParentOfZone(tree, zoneId);
    if (parentSplit) {
      splitPositioning = { dividerId: parentSplit.id, direction, zoneId };
    }
  }

  /**
   * Find the split node that directly contains a child with the given ID.
   */
  function findParentOfZone(node: ZoneTreeType, targetId: string): SplitNode | null {
    if (node.kind === 'leaf') return null;
    if (node.children[0].id === targetId || node.children[1].id === targetId) {
      return node;
    }
    return findParentOfZone(node.children[0], targetId) ?? findParentOfZone(node.children[1], targetId);
  }

  function handleSplitPositionMove(e: MouseEvent) {
    if (!splitPositioning) return;
    const el = findSplitElement(splitPositioning.dividerId);
    if (!el) return;

    const rect = el.getBoundingClientRect();
    let ratio: number;
    if (splitPositioning.direction === 'horizontal') {
      ratio = (e.clientX - rect.left) / rect.width;
    } else {
      ratio = (e.clientY - rect.top) / rect.height;
    }
    ratio = Math.max(0.1, Math.min(0.9, ratio));
    dispatch({ type: 'resize', dividerId: splitPositioning.dividerId, ratio });
  }

  function handleSplitPositionConfirm(e: MouseEvent) {
    if (!splitPositioning) return;
    // Apply final position
    handleSplitPositionMove(e);
    splitPositioning = null;
  }

  function handleSplitPositionKeyDown(e: KeyboardEvent) {
    if (!splitPositioning) return;
    if (e.key === 'Escape') {
      e.preventDefault();
      // Undo the split — join keeping the first (original) zone
      dispatch({ type: 'join', dividerId: splitPositioning.dividerId, keep: 'first' });
      splitPositioning = null;
    }
  }

  function handleZoneContextMenu(e: MouseEvent, zoneId: string, zoneTypeId: string) {
    const items: ContextMenuItem[] = [
      { label: 'Split Left / Right', action: () => performSplit(zoneId, 'horizontal') },
      { label: 'Split Top / Bottom', action: () => performSplit(zoneId, 'vertical') },
    ];
    contextMenu = { items, x: e.clientX, y: e.clientY };
  }

  function handleDividerContextMenu(e: MouseEvent, dividerId: string, direction: 'horizontal' | 'vertical') {
    const isHorizontal = direction === 'horizontal';
    const items: ContextMenuItem[] = [
      { label: isHorizontal ? 'Join → Keep Left' : 'Join → Keep Top', commandId: 'forge.zone.joinFirst', commandArgs: { dividerId } },
      { label: isHorizontal ? 'Join → Keep Right' : 'Join → Keep Bottom', commandId: 'forge.zone.joinSecond', commandArgs: { dividerId } },
      { label: '', commandId: '', separator: true },
      { label: 'Swap', commandId: 'forge.zone.swap', commandArgs: { dividerId } },
      { label: 'Reset Divider', commandId: 'forge.zone.resetDivider', commandArgs: { dividerId } },
    ];
    contextMenu = { items, x: e.clientX, y: e.clientY };
  }

  function closeContextMenu() {
    contextMenu = null;
  }

  // Attach/detach global listeners for split-positioning mode
  $effect(() => {
    if (!splitPositioning) return;

    const onMove = (e: MouseEvent) => handleSplitPositionMove(e);
    const onUp = (e: MouseEvent) => handleSplitPositionConfirm(e);
    const onKey = (e: KeyboardEvent) => handleSplitPositionKeyDown(e);

    window.addEventListener('mousemove', onMove);
    window.addEventListener('mouseup', onUp);
    window.addEventListener('keydown', onKey);

    return () => {
      window.removeEventListener('mousemove', onMove);
      window.removeEventListener('mouseup', onUp);
      window.removeEventListener('keydown', onKey);
    };
  });
</script>

{#if !ready}
  <div class="forge-loading">
    <p>Loading Urd Forge…</p>
  </div>
{:else}
  <div class="forge-app" class:forge-app--positioning={!!splitPositioning}>
    <GlobalMenuBar />
    <WorkspaceTabs />

    <div class="forge-app__content">
      {#if projectManager.isOpen}
        <ZoneRenderer
          node={workspaceManager.active.tree}
          zoneStates={workspaceManager.active.zoneStates}
          projectOpen={projectManager.isOpen}
          {dispatch}
          onZoneContextMenu={handleZoneContextMenu}
          onDividerContextMenu={handleDividerContextMenu}
        />
      {:else}
        <WelcomeScreen />
      {/if}
    </div>

    <GlobalStatusBar />
  </div>

  {#if contextMenu}
    <ContextMenu
      items={contextMenu.items}
      x={contextMenu.x}
      y={contextMenu.y}
      onClose={closeContextMenu}
    />
  {/if}
{/if}

<style>
  .forge-loading {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100vh;
    color: var(--forge-text-muted);
    font-size: var(--forge-font-size-md);
  }

  .forge-app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
  }

  .forge-app__content {
    flex: 1;
    overflow: hidden;
    position: relative;
  }

  .forge-app--positioning {
    cursor: crosshair;
  }

  .forge-app--positioning :global(*) {
    pointer-events: none !important;
  }

  .forge-app--positioning {
    pointer-events: auto !important;
  }
</style>
