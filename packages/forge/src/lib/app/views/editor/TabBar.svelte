<script lang="ts">
  /**
   * TabBar — horizontal tab strip for the Code Editor zone.
   *
   * Shows open file tabs with filename, dirty indicator, and close button.
   */

  export interface TabInfo {
    path: string;
    name: string;
    dirty: boolean;
  }

  interface Props {
    tabs: TabInfo[];
    activeTab: string | null;
    onSelectTab: (path: string) => void;
    onCloseTab: (path: string) => void;
  }

  let { tabs, activeTab, onSelectTab, onCloseTab }: Props = $props();

  function handleClose(e: MouseEvent, path: string): void {
    e.stopPropagation();
    onCloseTab(path);
  }

  function filename(path: string): string {
    const parts = path.split('/');
    return parts[parts.length - 1] || path;
  }
</script>

<div class="forge-tab-bar" role="tablist">
  {#each tabs as tab (tab.path)}
    <div
      class="forge-tab"
      class:forge-tab--active={tab.path === activeTab}
      role="tab"
      tabindex="0"
      aria-selected={tab.path === activeTab}
      onclick={() => onSelectTab(tab.path)}
      onkeydown={(e: KeyboardEvent) => { if (e.key === 'Enter') onSelectTab(tab.path); }}
    >
      <span class="forge-tab__name">{filename(tab.path)}</span>
      {#if tab.dirty}
        <span class="forge-tab__dirty" title="Unsaved changes">●</span>
      {/if}
      <button
        class="forge-tab__close"
        title="Close"
        onclick={(e: MouseEvent) => handleClose(e, tab.path)}
        aria-label="Close {filename(tab.path)}"
      >×</button>
    </div>
  {/each}
</div>

<style>
  .forge-tab-bar {
    display: flex;
    align-items: stretch;
    height: 28px;
    background-color: var(--forge-bg-secondary);
    border-bottom: 1px solid var(--forge-border-zone);
    overflow-x: auto;
    overflow-y: hidden;
    scrollbar-width: none;
  }

  .forge-tab-bar::-webkit-scrollbar {
    display: none;
  }

  .forge-tab {
    display: flex;
    align-items: center;
    gap: var(--forge-space-xs);
    padding: 0 var(--forge-space-md);
    border: none;
    border-right: 1px solid var(--forge-border-zone);
    background: transparent;
    color: var(--forge-text-secondary);
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-xs);
    cursor: pointer;
    white-space: nowrap;
    transition: background-color 0.1s, color 0.1s;
  }

  .forge-tab:hover {
    background-color: var(--forge-bg-tertiary);
    color: var(--forge-text-primary);
  }

  .forge-tab--active {
    background-color: var(--forge-bg-zone-viewport);
    color: var(--forge-text-primary);
    border-bottom: 1px solid var(--forge-bg-zone-viewport);
    margin-bottom: -1px;
  }

  .forge-tab__name {
    max-width: 160px;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .forge-tab__dirty {
    color: var(--forge-accent-primary);
    font-size: 8px;
    line-height: 1;
  }

  .forge-tab__close {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    height: 16px;
    padding: 0;
    border: none;
    border-radius: var(--forge-radius-sm);
    background: transparent;
    color: var(--forge-text-muted);
    font-size: 14px;
    line-height: 1;
    cursor: pointer;
    opacity: 0;
    transition: opacity 0.1s, background-color 0.1s;
  }

  .forge-tab:hover .forge-tab__close,
  .forge-tab--active .forge-tab__close {
    opacity: 1;
  }

  .forge-tab__close:hover {
    background-color: var(--forge-accent-primary);
    color: var(--forge-text-primary);
  }
</style>
