<script lang="ts">
  /**
   * EditorPanelContent — per-panel Svelte component for dockview integration.
   *
   * Each instance owns its own EditorPane (CodeMirror), Breadcrumb, extensions,
   * diagnostics subscription, and bus listeners. Mounted/unmounted by
   * DockviewEditorHost's IContentRenderer.
   */

  import { onMount, onDestroy } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { bufferMap } from '$lib/app/compiler/BufferMap';
  import { getCurrentTheme } from '$lib/framework/theme/ThemeEngine';
  import { appSettings } from '$lib/framework/settings/AppSettingsService';
  import { projectionRegistry } from '$lib/app/projections/ProjectionRegistry';
  import type { FileDiagnostics } from '$lib/app/projections/diagnostics-by-file';
  import type { DefinitionIndex } from '$lib/app/projections/definition-index';
  import { urdGotoDefinition, urdDefinitionLink, type DefinitionEntry, type DefinitionResolver } from './urd-navigation';
  import { urdHoverTooltip, type HoverDataProvider } from './urd-hover';
  import { createUrdCompletionSource, type AutocompleteDataProvider } from './urd-autocomplete';
  import { urdGutter, type GutterDataProvider } from './urd-gutter';
  import type { EntityRow } from '$lib/app/projections/entity-table';
  import type { PropertyRow } from '$lib/app/projections/property-table';
  import { autocompletion } from '@codemirror/autocomplete';
  import type { Extension } from '@codemirror/state';
  import type { FactSet, PropertyDependencyIndex } from '$lib/app/compiler/types';
  import { filesMatch, resolveCompilerPath } from './file-match';
  import EditorPane from './EditorPane.svelte';
  import Breadcrumb from './Breadcrumb.svelte';

  interface Props {
    filePath: string;
    /** Callback to open a file (for go-to-definition navigation). */
    onNavigate: (file: string, line: number, col?: number) => void;
    /** Callback when content changes (dirty tracking). */
    onContentChange?: (path: string, content: string) => void;
    /** Callback when cursor moves. */
    onCursorChange?: (path: string, line: number, col: number) => void;
  }

  let { filePath, onNavigate, onContentChange, onCursorChange }: Props = $props();

  let editorPane: EditorPane | undefined = $state();
  let currentTheme: 'gloaming' | 'parchment' = $state(getCurrentTheme());
  let cursorLine: number = $state(1);

  const unsubscribers: (() => void)[] = [];

  onMount(() => {
    // Load file content into the editor
    const content = bufferMap.get(filePath);
    if (content !== undefined && editorPane) {
      editorPane.setContent(content);
    }

    // Update diagnostics on mount
    updateDiagnostics();

    // Subscribe to theme changes
    unsubscribers.push(
      bus.subscribe('theme.changed', (payload) => {
        const { theme } = payload as { theme: string };
        currentTheme = theme as 'gloaming' | 'parchment';
      })
    );

    // Subscribe to compiler completion for diagnostics
    unsubscribers.push(
      bus.subscribe('compiler.completed', () => {
        updateDiagnostics();
      })
    );

    // Subscribe to settings changes
    unsubscribers.push(
      bus.subscribe('settings.changed', () => {
        // Reactivity handles this via $derived
      })
    );
  });

  onDestroy(() => {
    for (const unsub of unsubscribers) unsub();
  });

  // --- Diagnostics ---

  function updateDiagnostics(): void {
    if (!editorPane) return;
    const allDiags = projectionRegistry.get<FileDiagnostics[]>('urd.projection.diagnosticsByFile');
    if (!allDiags) {
      editorPane.clearDiagnostics();
      return;
    }
    const fileDiag = allDiags.find((fd: FileDiagnostics) => filesMatch(filePath, fd.file));
    if (fileDiag) {
      editorPane.setDiagnostics(fileDiag.diagnostics);
    } else {
      editorPane.clearDiagnostics();
    }
  }

  // --- Content change handler ---

  function handleContentChange(content: string): void {
    bufferMap.set(filePath, content);
    onContentChange?.(filePath, content);
  }

  function handleCursorChange(line: number, col: number): void {
    cursorLine = line;
    onCursorChange?.(filePath, line, col);
  }

  // --- Definition resolver ---

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
    const resolved = resolveCompilerPath(file, bufferMap.paths());
    onNavigate(resolved, line, col);
  }

  function getCurrentFile(): string | null {
    return filePath;
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
        const allDiags = projectionRegistry.get<FileDiagnostics[]>('urd.projection.diagnosticsByFile');
        const fileDiag = allDiags?.find((fd: FileDiagnostics) => filesMatch(filePath, fd.file));
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
      getCurrentFile() {
        return filePath;
      },
    };
  }

  // --- Extensions ---

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

  // --- Public API ---

  /** Scroll to a specific line. */
  export function scrollToLine(line: number, col?: number): void {
    editorPane?.scrollToLine(line, col);
  }

  /** Get the underlying EditorPane. */
  export function getEditorPane(): EditorPane | undefined {
    return editorPane;
  }

  /** Focus the editor. */
  export function focus(): void {
    editorPane?.focus();
  }
</script>

<div class="forge-editor-panel-content">
  <Breadcrumb
    {filePath}
    {cursorLine}
    {getDocText}
    onNavigate={handleBreadcrumbNavigate}
  />
  <div class="forge-editor-panel-content__editor">
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
  </div>
</div>

<style>
  .forge-editor-panel-content {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    overflow: hidden;
  }

  .forge-editor-panel-content__editor {
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }
</style>
