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
  import { workspaceManager } from '$lib/framework/workspace/WorkspaceManager';
  import { projectManager } from '$lib/framework/project/ProjectManager';
  import { bootstrap } from '$lib/app/bootstrap';
  import type { ContextMenuItem, ContextMenuTarget } from '$lib/framework/types';
  import type { ZoneTreeAction } from '$lib/framework/types';

  let ready = $state(false);
  let cleanup: (() => void) | null = null;

  // Context menu state
  let contextMenu = $state<{
    items: ContextMenuItem[];
    x: number;
    y: number;
  } | null>(null);

  onMount(async () => {
    cleanup = await bootstrap();
    ready = true;

    return () => {
      cleanup?.();
    };
  });

  function dispatch(action: ZoneTreeAction) {
    workspaceManager.dispatch(action);
  }

  function handleZoneContextMenu(e: MouseEvent, zoneId: string, zoneTypeId: string) {
    const items: ContextMenuItem[] = [
      { label: 'Split Horizontal', commandId: 'forge.zone.splitHorizontal', commandArgs: { zoneId } },
      { label: 'Split Vertical', commandId: 'forge.zone.splitVertical', commandArgs: { zoneId } },
    ];
    contextMenu = { items, x: e.clientX, y: e.clientY };
  }

  function handleDividerContextMenu(e: MouseEvent, dividerId: string) {
    const items: ContextMenuItem[] = [
      { label: 'Join → Keep First', commandId: 'forge.zone.joinFirst', commandArgs: { dividerId } },
      { label: 'Join → Keep Second', commandId: 'forge.zone.joinSecond', commandArgs: { dividerId } },
      { label: '', commandId: '', separator: true },
      { label: 'Swap', commandId: 'forge.zone.swap', commandArgs: { dividerId } },
      { label: 'Reset Divider', commandId: 'forge.zone.resetDivider', commandArgs: { dividerId } },
    ];
    contextMenu = { items, x: e.clientX, y: e.clientY };
  }

  function closeContextMenu() {
    contextMenu = null;
  }
</script>

{#if !ready}
  <div class="forge-loading">
    <p>Loading Urd Forge…</p>
  </div>
{:else}
  <div class="forge-app">
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
</style>
