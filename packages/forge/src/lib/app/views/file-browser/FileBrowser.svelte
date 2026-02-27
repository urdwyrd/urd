<script lang="ts">
  /**
   * FileBrowser — project file tree with dirty/diagnostic indicators.
   *
   * Reads project root from projectManager, lazy-expands directories via fileSystem.
   * Single-click opens file in editor. Right-click shows context menu.
   */

  import { onMount, onDestroy, untrack } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { projectManager } from '$lib/framework/project/ProjectManager.svelte';
  import { fileSystem } from '$lib/app/bootstrap';
  import { bufferMap } from '$lib/app/compiler/BufferMap';
  import { projectionRegistry } from '$lib/app/projections/ProjectionRegistry';
  import { navigationBroker } from '$lib/framework/navigation/NavigationBroker';
  import type { FileEntry } from '$lib/framework/filesystem/FileSystem';
  import type { FileDiagnostics } from '$lib/app/projections/diagnostics-by-file';
  import FileTreeNode from './FileTreeNode.svelte';

  interface FileBrowserState {
    expandedPaths: string[];
  }

  interface Props {
    zoneId: string;
    zoneTypeId: string;
    state: FileBrowserState | null;
    onStateChange: (newState: unknown) => void;
  }

  interface TreeItem {
    entry: FileEntry;
    depth: number;
    children?: TreeItem[];
    loaded?: boolean;
  }

  interface ContextMenuState {
    x: number;
    y: number;
    entry: FileEntry;
  }

  let { zoneId, zoneTypeId, state: zoneState, onStateChange }: Props = $props();

  let expandedPaths: Set<string> = $state(new Set(untrack(() => zoneState?.expandedPaths ?? [])));
  let selectedPath: string | null = $state(null);
  let rootItems: TreeItem[] = $state([]);
  let fileDiagnostics: FileDiagnostics[] = $state([]);
  let contextMenu: ContextMenuState | null = $state(null);

  const unsubscribers: (() => void)[] = [];

  // Portal action — moves element to document.body so fixed positioning
  // works correctly regardless of ancestor transforms.
  function portal(node: HTMLElement): { destroy: () => void } {
    document.body.appendChild(node);
    return {
      destroy() {
        node.remove();
      },
    };
  }

  onMount(async () => {
    if (projectManager.currentPath) {
      await loadDirectory(projectManager.currentPath, 0);
    }

    unsubscribers.push(
      bus.subscribe('project.opened', async (payload) => {
        const { path } = payload as { path: string };
        expandedPaths = new Set();
        selectedPath = null;
        await loadDirectory(path, 0);
      })
    );

    unsubscribers.push(
      bus.subscribe('project.closed', () => {
        rootItems = [];
        expandedPaths = new Set();
        selectedPath = null;
      })
    );

    unsubscribers.push(
      bus.subscribe('compiler.completed', () => {
        fileDiagnostics = projectionRegistry.get<FileDiagnostics[]>('urd.projection.diagnosticsByFile') ?? [];
      })
    );
  });

  onDestroy(() => {
    for (const unsub of unsubscribers) unsub();
    persistState();
  });

  function closeContextMenu(): void {
    contextMenu = null;
  }

  function persistState(): void {
    onStateChange({ expandedPaths: [...expandedPaths] } satisfies FileBrowserState);
  }

  async function loadDirectory(path: string, depth: number): Promise<TreeItem[]> {
    try {
      const entries = await fileSystem.listDirectory(path);
      entries.sort((a, b) => {
        if (a.isDirectory && !b.isDirectory) return -1;
        if (!a.isDirectory && b.isDirectory) return 1;
        return a.name.localeCompare(b.name);
      });

      const items: TreeItem[] = entries
        .filter((e) => !e.name.startsWith('.'))
        .map((entry) => ({
          entry,
          depth,
          children: entry.isDirectory ? [] : undefined,
          loaded: !entry.isDirectory,
        }));

      if (depth === 0) {
        rootItems = items;
      }
      return items;
    } catch (err) {
      console.error('Failed to list directory:', err);
      return [];
    }
  }

  async function toggleDirectory(entry: FileEntry): Promise<void> {
    if (expandedPaths.has(entry.path)) {
      expandedPaths.delete(entry.path);
      expandedPaths = new Set(expandedPaths);
    } else {
      expandedPaths.add(entry.path);
      expandedPaths = new Set(expandedPaths);
      await loadChildrenOf(entry.path);
    }
    persistState();
  }

  async function loadChildrenOf(dirPath: string): Promise<void> {
    const item = findItem(rootItems, dirPath);
    if (item && !item.loaded) {
      const entries = await fileSystem.listDirectory(dirPath);
      entries.sort((a, b) => {
        if (a.isDirectory && !b.isDirectory) return -1;
        if (!a.isDirectory && b.isDirectory) return 1;
        return a.name.localeCompare(b.name);
      });
      item.children = entries
        .filter((e) => !e.name.startsWith('.'))
        .map((e) => ({
          entry: e,
          depth: item.depth + 1,
          children: e.isDirectory ? [] : undefined,
          loaded: !e.isDirectory,
        }));
      item.loaded = true;
      rootItems = [...rootItems];
    }
  }

  function findItem(items: TreeItem[], path: string): TreeItem | null {
    for (const item of items) {
      if (item.entry.path === path) return item;
      if (item.children) {
        const found = findItem(item.children, path);
        if (found) return found;
      }
    }
    return null;
  }

  function getDiagnostics(file: string): { errors: number; warnings: number } {
    const fd = fileDiagnostics.find((d) => d.file === file);
    return { errors: fd?.errorCount ?? 0, warnings: fd?.warningCount ?? 0 };
  }

  function handleSelect(entry: FileEntry): void {
    selectedPath = entry.path;
  }

  function handleToggle(entry: FileEntry): void {
    toggleDirectory(entry);
  }

  function handleOpen(entry: FileEntry): void {
    if (entry.isFile) {
      navigationBroker.navigate({
        targetViewId: 'urd.codeEditor',
        params: { path: entry.path },
      });
    }
  }

  function handleNodeContextMenu(entry: FileEntry, event: MouseEvent): void {
    contextMenu = { x: event.clientX, y: event.clientY, entry };
  }

  // ── Context menu actions ──

  function ctxOpen(): void {
    if (!contextMenu) return;
    handleOpen(contextMenu.entry);
    contextMenu = null;
  }

  function ctxOpenInSourceViewer(): void {
    if (!contextMenu) return;
    navigationBroker.navigate({
      targetViewId: 'urd.sourceViewer',
      params: { path: contextMenu.entry.path },
    });
    contextMenu = null;
  }

  function ctxCopyPath(): void {
    if (!contextMenu) return;
    navigator.clipboard.writeText(contextMenu.entry.path);
    contextMenu = null;
  }

  function ctxCopyRelativePath(): void {
    if (!contextMenu || !projectManager.currentPath) return;
    const root = projectManager.currentPath;
    let rel = contextMenu.entry.path;
    if (rel.startsWith(root)) {
      rel = rel.slice(root.length);
      if (rel.startsWith('/') || rel.startsWith('\\')) rel = rel.slice(1);
    }
    navigator.clipboard.writeText(rel);
    contextMenu = null;
  }

  function ctxCopyName(): void {
    if (!contextMenu) return;
    navigator.clipboard.writeText(contextMenu.entry.name);
    contextMenu = null;
  }

  function ctxRevealInExplorer(): void {
    if (!contextMenu) return;
    const entry = contextMenu.entry;
    contextMenu = null;
    if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
      import('@tauri-apps/api/core').then(({ invoke }) => {
        invoke('reveal_in_explorer', { path: entry.path });
      }).catch(() => {});
    }
  }

  // Flatten tree for rendering
  let flatItems = $derived.by(() => {
    const result: TreeItem[] = [];
    function flatten(items: TreeItem[]): void {
      for (const item of items) {
        result.push(item);
        if (item.entry.isDirectory && expandedPaths.has(item.entry.path) && item.children) {
          flatten(item.children);
        }
      }
    }
    flatten(rootItems);
    return result;
  });
</script>

<div class="forge-file-browser" role="tree">
  {#if flatItems.length === 0}
    <div class="forge-file-browser__empty">
      {#if projectManager.isOpen}
        <p>Empty project</p>
      {:else}
        <p>No project open</p>
        <p class="forge-file-browser__hint">Open a project to browse files</p>
      {/if}
    </div>
  {:else}
    <div class="forge-file-browser__list">
      {#each flatItems as item (item.entry.path)}
        {@const diags = getDiagnostics(item.entry.path)}
        <FileTreeNode
          entry={item.entry}
          depth={item.depth}
          expanded={expandedPaths.has(item.entry.path)}
          selected={selectedPath === item.entry.path}
          dirty={bufferMap.isDirty(item.entry.path)}
          errorCount={diags.errors}
          warningCount={diags.warnings}
          onSelect={handleSelect}
          onToggle={handleToggle}
          onOpen={handleOpen}
          onContextMenu={handleNodeContextMenu}
        />
      {/each}
    </div>
  {/if}
</div>

<!-- Context menu — portaled to document.body for correct fixed positioning -->
{#if contextMenu}
  <div class="forge-context-menu-portal" use:portal>
    <!-- Invisible backdrop catches clicks/right-clicks to dismiss -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="forge-context-menu-backdrop"
      role="presentation"
      onclick={closeContextMenu}
      oncontextmenu={(e) => { e.preventDefault(); closeContextMenu(); }}
    ></div>
    <div
      class="forge-context-menu"
      style="left: {contextMenu.x}px; top: {contextMenu.y}px"
      role="menu"
    >
      {#if contextMenu.entry.isFile}
        <button class="forge-context-menu__item" type="button" role="menuitem" onclick={ctxOpen}>
          Open
        </button>
        <button class="forge-context-menu__item" type="button" role="menuitem" onclick={ctxOpenInSourceViewer}>
          Open in Source Viewer
        </button>
        <div class="forge-context-menu__sep"></div>
      {/if}
      <button class="forge-context-menu__item" type="button" role="menuitem" onclick={ctxCopyPath}>
        Copy Path
      </button>
      <button class="forge-context-menu__item" type="button" role="menuitem" onclick={ctxCopyRelativePath}>
        Copy Relative Path
      </button>
      <button class="forge-context-menu__item" type="button" role="menuitem" onclick={ctxCopyName}>
        Copy Name
      </button>
      <div class="forge-context-menu__sep"></div>
      <button class="forge-context-menu__item" type="button" role="menuitem" onclick={ctxRevealInExplorer}>
        Reveal in File Explorer
      </button>
    </div>
  </div>
{/if}

<style>
  .forge-file-browser {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    overflow-y: auto;
    overflow-x: hidden;
  }

  .forge-file-browser__empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    color: var(--forge-text-muted);
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-md);
  }

  .forge-file-browser__hint {
    font-size: var(--forge-font-size-sm);
    margin-top: var(--forge-space-sm);
  }

  .forge-file-browser__list {
    padding: var(--forge-space-xs) 0;
  }

  /* ── Context menu (portaled to body) ── */

  .forge-context-menu-backdrop {
    position: fixed;
    inset: 0;
    z-index: 9998;
  }

  .forge-context-menu {
    position: fixed;
    z-index: 9999;
    min-width: 200px;
    padding: var(--forge-space-xs) 0;
    background-color: var(--forge-bg-secondary);
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-md);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-sm);
  }

  .forge-context-menu__item {
    display: flex;
    align-items: center;
    width: 100%;
    height: 28px;
    padding: 0 var(--forge-space-lg);
    background: none;
    border: none;
    color: var(--forge-text-primary);
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-sm);
    text-align: left;
    cursor: pointer;
  }

  .forge-context-menu__item:hover {
    background-color: var(--forge-accent-selection);
  }

  .forge-context-menu__sep {
    height: 1px;
    margin: var(--forge-space-xs) 0;
    background-color: var(--forge-border-zone);
  }
</style>
