<script lang="ts">
  /**
   * FileTreeNode — single row in the file tree: indent, icon, name, badges.
   *
   * Single-click opens files in editor (like VS Code).
   * Double-click pins the tab. Right-click shows context menu.
   */

  import type { FileEntry } from '$lib/framework/filesystem/FileSystem';

  interface Props {
    entry: FileEntry;
    depth: number;
    expanded: boolean;
    selected: boolean;
    dirty: boolean;
    errorCount: number;
    warningCount: number;
    onSelect: (entry: FileEntry) => void;
    onToggle: (entry: FileEntry) => void;
    onOpen: (entry: FileEntry) => void;
    onContextMenu: (entry: FileEntry, event: MouseEvent) => void;
  }

  let { entry, depth, expanded, selected, dirty, errorCount, warningCount, onSelect, onToggle, onOpen, onContextMenu }: Props = $props();

  let icon = $derived(entry.isDirectory ? (expanded ? '\u25BE' : '\u25B8') : '\u25C7');

  function handleClick(): void {
    onSelect(entry);
    if (entry.isDirectory) {
      onToggle(entry);
    } else {
      // Single-click opens file in editor (like VS Code)
      onOpen(entry);
    }
  }

  function handleDoubleClick(): void {
    if (entry.isDirectory) {
      onToggle(entry);
    }
    // For files, double-click could pin the tab (future enhancement)
  }

  function handleContextMenu(e: MouseEvent): void {
    e.preventDefault();
    onSelect(entry);
    onContextMenu(entry, e);
  }

  function handleKeydown(e: KeyboardEvent): void {
    if (e.key === 'Enter') {
      if (entry.isFile) {
        onOpen(entry);
      } else {
        onToggle(entry);
      }
    } else if (e.key === 'ArrowRight' && entry.isDirectory && !expanded) {
      onToggle(entry);
    } else if (e.key === 'ArrowLeft' && entry.isDirectory && expanded) {
      onToggle(entry);
    }
  }
</script>

<div
  class="forge-file-tree-node"
  class:forge-file-tree-node--selected={selected}
  class:forge-file-tree-node--directory={entry.isDirectory}
  role="treeitem"
  tabindex="0"
  aria-selected={selected}
  aria-expanded={entry.isDirectory ? expanded : undefined}
  style:padding-left="{depth * 16 + 8}px"
  onclick={handleClick}
  ondblclick={handleDoubleClick}
  oncontextmenu={handleContextMenu}
  onkeydown={handleKeydown}
>
  <span class="forge-file-tree-node__icon">{icon}</span>
  <span class="forge-file-tree-node__name">{entry.name}</span>
  {#if dirty}
    <span class="forge-file-tree-node__badge forge-file-tree-node__badge--dirty" title="Unsaved changes">{'\u25CF'}</span>
  {/if}
  {#if errorCount > 0}
    <span class="forge-file-tree-node__badge forge-file-tree-node__badge--error" title="{errorCount} error{errorCount > 1 ? 's' : ''}">{errorCount}</span>
  {/if}
  {#if warningCount > 0}
    <span class="forge-file-tree-node__badge forge-file-tree-node__badge--warning" title="{warningCount} warning{warningCount > 1 ? 's' : ''}">{warningCount}</span>
  {/if}
</div>

<style>
  .forge-file-tree-node {
    display: flex;
    align-items: center;
    gap: var(--forge-space-xs);
    height: 24px;
    cursor: pointer;
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-sm);
    color: var(--forge-text-primary);
    white-space: nowrap;
    user-select: none;
  }

  .forge-file-tree-node:hover {
    background-color: var(--forge-table-row-hover);
  }

  .forge-file-tree-node--selected {
    background-color: var(--forge-accent-selection);
  }

  .forge-file-tree-node:focus-visible {
    outline: 1px solid var(--forge-accent-primary);
    outline-offset: -1px;
  }

  .forge-file-tree-node__icon {
    flex-shrink: 0;
    width: 14px;
    text-align: center;
    color: var(--forge-text-muted);
    font-size: var(--forge-font-size-xs);
  }

  .forge-file-tree-node--directory .forge-file-tree-node__icon {
    color: var(--forge-accent-secondary);
  }

  .forge-file-tree-node__name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .forge-file-tree-node__badge {
    flex-shrink: 0;
    padding: 0 4px;
    border-radius: var(--forge-radius-sm);
    font-size: 9px;
    font-weight: 600;
    line-height: 16px;
  }

  .forge-file-tree-node__badge--dirty {
    color: var(--forge-accent-primary);
  }

  .forge-file-tree-node__badge--error {
    background-color: var(--forge-status-error);
    color: white;
  }

  .forge-file-tree-node__badge--warning {
    background-color: var(--forge-status-warning);
    color: black;
  }
</style>
