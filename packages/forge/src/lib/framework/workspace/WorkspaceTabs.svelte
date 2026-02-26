<script lang="ts">
  /**
   * Workspace tab bar — Blender-style workspace switching.
   */

  import { workspaceManager } from './WorkspaceManager';

  let editingIndex = $state<number | null>(null);
  let editingName = $state('');

  function startRename(index: number) {
    editingIndex = index;
    editingName = workspaceManager.workspaces[index].name;
  }

  function finishRename() {
    if (editingIndex !== null && editingName.trim()) {
      workspaceManager.rename(editingIndex, editingName.trim());
    }
    editingIndex = null;
  }

  function cancelRename() {
    editingIndex = null;
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Enter') finishRename();
    if (e.key === 'Escape') cancelRename();
  }

  function handleAddWorkspace() {
    workspaceManager.create(`Workspace ${workspaceManager.workspaces.length + 1}`);
  }
</script>

<div class="forge-workspace-tabs">
  <div class="forge-workspace-tabs__list" role="tablist">
    {#each workspaceManager.workspaces as ws, i}
      <div
        class="forge-workspace-tabs__tab"
        class:forge-workspace-tabs__tab--active={i === workspaceManager.activeIndex}
        role="tab"
        aria-selected={i === workspaceManager.activeIndex}
      >
        {#if editingIndex === i}
          <input
            class="forge-workspace-tabs__rename forge-selectable"
            bind:value={editingName}
            onblur={finishRename}
            onkeydown={handleKeyDown}
          />
        {:else}
          <button
            class="forge-workspace-tabs__btn"
            onclick={() => workspaceManager.activate(i)}
            ondblclick={() => startRename(i)}
          >
            {ws.name}
          </button>
        {/if}

        {#if workspaceManager.workspaces.length > 1}
          <button
            class="forge-workspace-tabs__close"
            onclick={() => workspaceManager.remove(i)}
            title="Close workspace"
          >
            ×
          </button>
        {/if}
      </div>
    {/each}
  </div>

  <button
    class="forge-workspace-tabs__add"
    onclick={handleAddWorkspace}
    title="New workspace"
  >
    +
  </button>
</div>

<style>
  .forge-workspace-tabs {
    display: flex;
    align-items: center;
    height: 28px;
    padding: 0 var(--forge-space-sm);
    background: var(--forge-bg-primary);
    border-bottom: 1px solid var(--forge-border-zone);
    flex-shrink: 0;
    gap: var(--forge-space-xs);
  }

  .forge-workspace-tabs__list {
    display: flex;
    align-items: center;
    gap: 1px;
    overflow-x: auto;
    scrollbar-width: none;
  }

  .forge-workspace-tabs__list::-webkit-scrollbar {
    display: none;
  }

  .forge-workspace-tabs__tab {
    display: flex;
    align-items: center;
    height: 24px;
    border-radius: var(--forge-radius-sm) var(--forge-radius-sm) 0 0;
    background: transparent;
    transition: background 0.1s;
  }

  .forge-workspace-tabs__tab--active {
    background: var(--forge-bg-secondary);
  }

  .forge-workspace-tabs__btn {
    padding: 0 var(--forge-space-md);
    border: none;
    background: transparent;
    color: var(--forge-text-secondary);
    font-size: var(--forge-font-size-sm);
    cursor: pointer;
    white-space: nowrap;
    height: 100%;
  }

  .forge-workspace-tabs__tab--active .forge-workspace-tabs__btn {
    color: var(--forge-text-primary);
  }

  .forge-workspace-tabs__btn:hover {
    color: var(--forge-text-primary);
  }

  .forge-workspace-tabs__close {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    height: 16px;
    border: none;
    background: transparent;
    color: var(--forge-text-muted);
    font-size: 14px;
    cursor: pointer;
    border-radius: var(--forge-radius-sm);
    margin-right: var(--forge-space-xs);
  }

  .forge-workspace-tabs__close:hover {
    background: var(--forge-bg-tertiary);
    color: var(--forge-text-primary);
  }

  .forge-workspace-tabs__rename {
    width: 100px;
    height: 20px;
    padding: 0 var(--forge-space-sm);
    border: 1px solid var(--forge-accent-primary);
    border-radius: var(--forge-radius-sm);
    background: var(--forge-bg-zone-viewport);
    color: var(--forge-text-primary);
    font-size: var(--forge-font-size-sm);
    outline: none;
  }

  .forge-workspace-tabs__add {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    border: none;
    border-radius: var(--forge-radius-sm);
    background: transparent;
    color: var(--forge-text-muted);
    font-size: 16px;
    cursor: pointer;
  }

  .forge-workspace-tabs__add:hover {
    background: var(--forge-bg-secondary);
    color: var(--forge-text-primary);
  }
</style>
