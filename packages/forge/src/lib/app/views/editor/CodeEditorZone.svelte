<script lang="ts">
  /**
   * CodeEditorZone — singleton zone for the Urd code editor.
   *
   * Manages tabs, file buffers, cursor state, and compilation integration.
   * Uses EditorPane for the CodeMirror instance and TabBar for tab navigation.
   */

  import { onMount, onDestroy, tick } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { bufferMap } from '$lib/app/compiler/BufferMap';
  import { getCurrentTheme } from '$lib/framework/theme/ThemeEngine';
  import { appSettings } from '$lib/framework/settings/AppSettingsService';
  import { fileSystem } from '$lib/app/bootstrap';
  import { navigationBroker } from '$lib/framework/navigation/NavigationBroker';
  import { projectionRegistry } from '$lib/app/projections/ProjectionRegistry';
  import type { FileDiagnostics } from '$lib/app/projections/diagnostics-by-file';
  import type { DefinitionIndex } from '$lib/app/projections/definition-index';
  import { urdGotoDefinition, urdDefinitionLink, type DefinitionEntry, type DefinitionResolver } from './urd-navigation';
  import { urdHoverTooltip, type HoverDataProvider } from './urd-hover';
  import { createUrdCompletionSource, type AutocompleteDataProvider } from './urd-autocomplete';
  import { urdGutter, type GutterDataProvider } from './urd-gutter';
  import type { EntityRow } from '$lib/app/projections/entity-table';
  import type { PropertyRow } from '$lib/app/projections/property-table';
  import type { DefinitionIndex as DefIdx } from '$lib/app/projections/definition-index';
  import { autocompletion } from '@codemirror/autocomplete';
  import type { Extension } from '@codemirror/state';
  import type { FactSet, PropertyDependencyIndex, Diagnostic } from '$lib/app/compiler/types';
  import { filesMatch, resolveCompilerPath } from './file-match';
  import TabBar, { type TabInfo } from './TabBar.svelte';
  import EditorPane from './EditorPane.svelte';
  import Breadcrumb from './Breadcrumb.svelte';

  interface TabState {
    path: string;
    cursorLine: number;
    cursorCol: number;
    scrollTop: number;
  }

  interface CodeEditorState {
    openTabs: TabState[];
    activeTab: string | null;
  }

  interface Props {
    zoneId: string;
    zoneTypeId: string;
    state: CodeEditorState | null;
    onStateChange: (newState: unknown) => void;
  }

  let { zoneId, zoneTypeId, state: zoneState, onStateChange }: Props = $props();

  let editorPane: EditorPane | undefined = $state();
  let currentTheme: 'gloaming' | 'parchment' = $state(getCurrentTheme());
  let activeTabPath: string | null = $state(null);
  let openTabs: TabState[] = $state([]);
  let cursorLine: number = $state(1);
  let tabInfos: TabInfo[] = $derived(
    openTabs.map((t: TabState) => ({
      path: t.path,
      name: t.path.split('/').pop() || t.path,
      dirty: bufferMap.isDirty(t.path),
    }))
  );

  const unsubscribers: (() => void)[] = [];

  // Restore state or initialise
  onMount(() => {
    if (zoneState) {
      const s = zoneState;
      openTabs = s.openTabs ?? [];
      activeTabPath = s.activeTab ?? null;
    }

    // Tabs are opened on navigation (openFile) — no auto-open of all buffers.

    // Load active tab content after mount, then check for pending navigation
    tick().then(() => {
      loadActiveTab();
      publishActiveFile(activeTabPath);
      // Handle navigation that arrived before this component mounted
      const pending = navigationBroker.consumePendingParams(zoneId);
      if (pending?.viewId === 'urd.codeEditor' && pending.params.path) {
        openFile(
          pending.params.path as string,
          pending.params.line as number | undefined,
          pending.params.col as number | undefined,
        );
      }
    });

    // Subscribe to theme changes
    unsubscribers.push(
      bus.subscribe('theme.changed', (payload) => {
        const { theme } = payload as { theme: string };
        currentTheme = theme as 'gloaming' | 'parchment';
      })
    );

    // Subscribe to compiler completion for diagnostics and dirty state
    unsubscribers.push(
      bus.subscribe('compiler.completed', () => {
        // Trigger reactivity on dirty state
        openTabs = [...openTabs];
        // Push diagnostics for the active file
        updateDiagnostics();
      })
    );

    // Subscribe to settings changes for editor configuration
    unsubscribers.push(
      bus.subscribe('settings.changed', () => {
        // Reactivity handles this via $derived
      })
    );

    // Subscribe to navigation — open file when navigated to this zone
    unsubscribers.push(
      bus.subscribe('navigation.completed', (payload) => {
        const { zoneId: targetZoneId, viewId, params } = payload as {
          zoneId: string;
          viewId: string;
          params: Record<string, unknown>;
        };
        if (targetZoneId === zoneId && viewId === 'urd.codeEditor' && params.path) {
          openFile(
            params.path as string,
            params.line as number | undefined,
            params.col as number | undefined,
          );
        }
      })
    );

    // Subscribe to editor command bus signals
    unsubscribers.push(
      bus.subscribe('editor.closeTab', () => closeActiveTab())
    );
    unsubscribers.push(
      bus.subscribe('editor.nextTab', () => nextTab())
    );
    unsubscribers.push(
      bus.subscribe('editor.prevTab', () => prevTab())
    );
    unsubscribers.push(
      bus.subscribe('editor.save', () => saveActiveFile())
    );
  });

  onDestroy(() => {
    for (const unsub of unsubscribers) unsub();
    persistState();
  });

  function loadActiveTab(): void {
    if (!editorPane || !activeTabPath) return;
    const content = bufferMap.get(activeTabPath);
    if (content !== undefined) {
      editorPane.setContent(content);
      // Restore cursor and scroll
      const tabState = openTabs.find((t: TabState) => t.path === activeTabPath);
      if (tabState) {
        editorPane.scrollToLine(tabState.cursorLine, tabState.cursorCol);
        // Delay scroll restoration for layout to settle
        requestAnimationFrame(() => {
          editorPane?.setScrollTop(tabState.scrollTop);
        });
      }
      // Refresh diagnostics for the newly loaded file
      updateDiagnostics();
    }
  }

  function saveTabState(): void {
    if (!editorPane || !activeTabPath) return;
    const tab = openTabs.find((t: TabState) => t.path === activeTabPath);
    if (tab) {
      const cursor = editorPane.getCursorPosition();
      tab.cursorLine = cursor.line;
      tab.cursorCol = cursor.col;
      tab.scrollTop = editorPane.getScrollTop();
    }
  }

  function persistState(): void {
    saveTabState();
    const editorState: CodeEditorState = {
      openTabs: openTabs.map((t: TabState) => ({ ...t })),
      activeTab: activeTabPath,
    };
    onStateChange(editorState);
  }

  function publishActiveFile(path: string | null): void {
    if (bus.hasChannel('editor.activeFile')) {
      bus.publish('editor.activeFile', { path });
    }
  }

  function selectTab(path: string): void {
    if (path === activeTabPath) return;
    saveTabState();
    activeTabPath = path;
    publishActiveFile(path);
    tick().then(() => loadActiveTab());
    persistState();
  }

  function closeTab(path: string): void {
    saveTabState();
    const idx = openTabs.findIndex((t: TabState) => t.path === path);
    if (idx === -1) return;

    openTabs = openTabs.filter((t: TabState) => t.path !== path);

    if (activeTabPath === path) {
      // Switch to adjacent tab
      if (openTabs.length > 0) {
        const newIdx = Math.min(idx, openTabs.length - 1);
        activeTabPath = openTabs[newIdx].path;
        tick().then(() => loadActiveTab());
      } else {
        activeTabPath = null;
        editorPane?.setContent('');
      }
    }

    persistState();
  }

  function handleContentChange(content: string): void {
    if (activeTabPath) {
      bufferMap.set(activeTabPath, content);
      // Trigger tab dirty state reactivity
      openTabs = [...openTabs];
    }
  }

  function handleCursorChange(line: number, col: number): void {
    cursorLine = line;
    if (!activeTabPath) return;
    const tab = openTabs.find((t: TabState) => t.path === activeTabPath);
    if (tab) {
      tab.cursorLine = line;
      tab.cursorCol = col;
    }
  }

  /** Open a file in the editor — called externally via navigation. */
  export function openFile(path: string, line?: number, col?: number): void {
    // Resolve compiler short filenames (e.g. "sunken-citadel.urd.md") to full
    // buffer paths so we match existing tabs instead of opening duplicates.
    const resolved = resolveCompilerPath(path, bufferMap.paths());
    path = resolved;

    let tab = openTabs.find((t: TabState) => t.path === path);
    if (!tab) {
      tab = { path, cursorLine: line ?? 1, cursorCol: col ?? 1, scrollTop: 0 };
      openTabs = [...openTabs, tab];
    } else if (line) {
      tab.cursorLine = line;
      tab.cursorCol = col ?? 1;
    }

    if (activeTabPath !== path) {
      saveTabState();
      activeTabPath = path;
      publishActiveFile(path);
      tick().then(() => loadActiveTab());
    } else if (line) {
      editorPane?.scrollToLine(line, col);
    }

    persistState();
  }

  /** Close the active tab — used by Ctrl+W command. */
  export function closeActiveTab(): void {
    if (activeTabPath) {
      closeTab(activeTabPath);
    }
  }

  /** Switch to the next tab. */
  export function nextTab(): void {
    if (openTabs.length <= 1) return;
    const idx = openTabs.findIndex((t: TabState) => t.path === activeTabPath);
    const nextIdx = (idx + 1) % openTabs.length;
    selectTab(openTabs[nextIdx].path);
  }

  /** Switch to the previous tab. */
  export function prevTab(): void {
    if (openTabs.length <= 1) return;
    const idx = openTabs.findIndex((t: TabState) => t.path === activeTabPath);
    const prevIdx = (idx - 1 + openTabs.length) % openTabs.length;
    selectTab(openTabs[prevIdx].path);
  }

  /** Update diagnostics for the active file from the projection registry. */
  function updateDiagnostics(): void {
    if (!editorPane || !activeTabPath) return;
    const allDiags = projectionRegistry.get<FileDiagnostics[]>('urd.projection.diagnosticsByFile');
    if (!allDiags) {
      editorPane.clearDiagnostics();
      return;
    }
    // The Rust compiler uses short filenames (e.g., "main.urd.md") in diagnostic
    // spans, while buffer/tab paths are full paths. Match by suffix.
    const fileDiag = allDiags.find((fd: FileDiagnostics) => filesMatch(activeTabPath!, fd.file));
    if (fileDiag) {
      editorPane.setDiagnostics(fileDiag.diagnostics);
    } else {
      editorPane.clearDiagnostics();
    }
  }

  /** Save the active file to disk. */
  async function saveActiveFile(): Promise<void> {
    if (!activeTabPath || !fileSystem) return;
    const content = bufferMap.get(activeTabPath);
    if (content === undefined) return;

    try {
      await fileSystem.writeFile(activeTabPath, content);
      bufferMap.markClean(activeTabPath);
      openTabs = [...openTabs]; // Trigger dirty state reactivity
    } catch (err) {
      console.error('Failed to save file:', err);
    }
  }

  /** Get the EditorPane instance for external access. */
  export function getEditorPane(): EditorPane | undefined {
    return editorPane;
  }

  // --- Definition resolver for go-to-definition ---

  function getDefinitionResolver(): DefinitionResolver | null {
    const index = projectionRegistry.get<DefinitionIndex>('urd.projection.definitionIndex');
    if (!index) return null;

    return {
      findByKey(key: string): DefinitionEntry | null {
        return index.find((e: DefinitionEntry) => e.key === key) ?? null;
      },
      find(predicate: (entry: DefinitionEntry) => boolean): DefinitionEntry | null {
        return index.find(predicate) ?? null;
      },
      getEntityTypeName(entityId: string): string | null {
        const raw = projectionRegistry.get<Record<string, unknown>>('urd.projection.rawUrdJson');
        const entities = raw?.entities as Record<string, Record<string, unknown>> | undefined;
        if (entities?.[entityId]) {
          return (entities[entityId].type as string) ?? null;
        }
        return null;
      },
    };
  }

  function handleNavigate(file: string, line: number, col?: number): void {
    // Resolve compiler short filename to full buffer path
    const resolved = resolveCompilerPath(file, bufferMap.paths());
    openFile(resolved, line, col);
  }

  function getCurrentFile(): string | null {
    return activeTabPath;
  }

  // --- Hover data provider ---

  function getHoverProvider(): HoverDataProvider | null {
    return {
      getParsedWorld() {
        return projectionRegistry.get<Record<string, unknown>>('urd.projection.rawUrdJson') ?? null;
      },
      getDefinitionIndex() {
        return projectionRegistry.get<DefinitionIndex>('urd.projection.definitionIndex') ?? null;
      },
      getFactSet() {
        return projectionRegistry.get<FactSet>('urd.projection.factSet') ?? null;
      },
      getPropertyDependencyIndex() {
        return projectionRegistry.get<PropertyDependencyIndex>('urd.projection.propertyDependencyIndex') ?? null;
      },
      getDiagnostics() {
        if (!activeTabPath) return null;
        const allDiags = projectionRegistry.get<FileDiagnostics[]>('urd.projection.diagnosticsByFile');
        const fileDiag = allDiags?.find((fd: FileDiagnostics) => filesMatch(activeTabPath!, fd.file));
        return fileDiag?.diagnostics ?? null;
      },
      getEntityTable() {
        return projectionRegistry.get<EntityRow[]>('urd.projection.entityTable') ?? null;
      },
      getPropertyTable() {
        return projectionRegistry.get<PropertyRow[]>('urd.projection.propertyTable') ?? null;
      },
    };
  }

  // --- Autocomplete data provider ---

  function getAutocompleteProvider(): AutocompleteDataProvider | null {
    return {
      getWorldData() {
        return projectionRegistry.get('urd.projection.urdJson') ?? null;
      },
      getDefinitionIndex() {
        return projectionRegistry.get<DefinitionIndex>('urd.projection.definitionIndex') ?? null;
      },
    };
  }

  // --- Breadcrumb helpers ---

  function getDocText(): string | null {
    return editorPane?.getContent() ?? null;
  }

  function handleBreadcrumbNavigate(line: number): void {
    editorPane?.scrollToLine(line);
  }

  // --- Gutter data provider ---

  function getGutterProvider(): GutterDataProvider | null {
    return {
      getDiagnosticsByFile() {
        return projectionRegistry.get<FileDiagnostics[]>('urd.projection.diagnosticsByFile');
      },
      getDefinitionIndex() {
        return projectionRegistry.get<DefIdx>('urd.projection.definitionIndex');
      },
      getCurrentFile() {
        return activeTabPath;
      },
    };
  }

  // --- Extra extensions for the editor ---

  const editorExtensions: Extension[] = [
    urdGotoDefinition(getDefinitionResolver, handleNavigate, getCurrentFile),
    ...urdDefinitionLink(getDefinitionResolver),
    urdHoverTooltip(getHoverProvider),
    autocompletion({ override: [createUrdCompletionSource(getAutocompleteProvider)] }),
    urdGutter(getGutterProvider),
  ];

  // Settings-derived values
  let editorTabSize = $derived(appSettings.get('editorTabSize'));
  let editorLineNumbers = $derived(appSettings.get('editorLineNumbers'));
  let editorWordWrap = $derived(appSettings.get('editorWordWrap'));
</script>

<div class="forge-code-editor-zone">
  {#if openTabs.length > 0}
    <TabBar
      tabs={tabInfos}
      activeTab={activeTabPath}
      onSelectTab={selectTab}
      onCloseTab={closeTab}
    />
  {/if}

  {#if activeTabPath}
    <Breadcrumb
      filePath={activeTabPath}
      {cursorLine}
      {getDocText}
      onNavigate={handleBreadcrumbNavigate}
    />
  {/if}

  <div class="forge-code-editor-zone__editor">
    {#if activeTabPath}
      <EditorPane
        bind:this={editorPane}
        theme={currentTheme}
        tabSize={editorTabSize}
        showLineNumbers={editorLineNumbers}
        wordWrap={editorWordWrap}
        extraExtensions={editorExtensions}
        onContentChange={handleContentChange}
        onCursorChange={handleCursorChange}
      />
    {:else}
      <div class="forge-code-editor-zone__empty">
        <p>No files open</p>
        <p class="forge-code-editor-zone__hint">Open a project to get started</p>
      </div>
    {/if}
  </div>
</div>

<style>
  .forge-code-editor-zone {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    overflow: hidden;
  }

  .forge-code-editor-zone__editor {
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }

  .forge-code-editor-zone__empty {
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

  .forge-code-editor-zone__hint {
    font-size: var(--forge-font-size-sm);
    margin-top: var(--forge-space-sm);
  }
</style>
