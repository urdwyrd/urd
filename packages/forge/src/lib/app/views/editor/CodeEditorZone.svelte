<script lang="ts">
  /**
   * CodeEditorZone — singleton zone for the Urd code editor.
   *
   * Uses dockview-core to manage editor groups with drag-and-drop tab support.
   * Each open file gets its own dockview panel with an independent CodeMirror
   * instance (EditorPanelContent), preserving undo history across tab switches.
   */

  import { onMount, onDestroy, tick } from 'svelte';
  import { createDockview, type DockviewApi, type SerializedDockview } from 'dockview-core';
  import 'dockview-core/dist/styles/dockview.css';
  import './dockview-forge.css';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { bufferMap } from '$lib/app/compiler/BufferMap';
  import { fileSystem } from '$lib/app/bootstrap';
  import { navigationBroker } from '$lib/framework/navigation/NavigationBroker';
  import { resolveCompilerPath } from './file-match';
  import { EditorPanelRenderer, EditorTabRenderer, getRenderer, type EditorPanelParams } from './DockviewEditorHost';

  /** Dockview state persisted to the zone. */
  interface CodeEditorState {
    dockviewLayout: SerializedDockview | null;
  }

  interface Props {
    zoneId: string;
    zoneTypeId: string;
    state: CodeEditorState | null;
    onStateChange: (newState: unknown) => void;
  }

  let { zoneId, zoneTypeId, state: zoneState, onStateChange }: Props = $props();

  let containerEl: HTMLDivElement | undefined = $state();
  let dockviewApi: DockviewApi | undefined;
  let hasAnyPanels = $state(false);

  const unsubscribers: (() => void)[] = [];

  // --- Panel ID convention ---

  function panelId(path: string): string {
    return `panel_${path}`;
  }

  // --- Navigation callback shared across all panels ---

  function handleNavigate(file: string, line: number, col?: number): void {
    openFile(file, line, col);
  }

  // --- Dockview lifecycle ---

  onMount(() => {
    if (!containerEl) return;

    const forgeTheme = {
      name: 'forge',
      className: 'forge-dockview-theme',
    };

    dockviewApi = createDockview(containerEl, {
      theme: forgeTheme,
      createComponent: (options) => {
        if (options.name === 'editorPanel') {
          return new EditorPanelRenderer(handleNavigate);
        }
        throw new Error(`Unknown dockview component: ${options.name}`);
      },
      createTabComponent: (options) => {
        if (options.name === 'editorTab') {
          return new EditorTabRenderer();
        }
        return undefined;
      },
      disableFloatingGroups: true,
    });

    // Track panel count for empty-state overlay
    dockviewApi.onDidAddPanel(() => {
      hasAnyPanels = (dockviewApi?.totalPanels ?? 0) > 0;
    });
    dockviewApi.onDidRemovePanel(() => {
      hasAnyPanels = (dockviewApi?.totalPanels ?? 0) > 0;
      persistState();
    });

    // Publish active file when active panel changes
    dockviewApi.onDidActivePanelChange((panel) => {
      const path = panel ? (panel.params as EditorPanelParams | undefined)?.path ?? null : null;
      publishActiveFile(path);
      persistState();
    });

    // Persist on layout changes (group splits, moves)
    dockviewApi.onDidMovePanel(() => persistState());

    // Restore saved layout or start empty
    const savedLayout = zoneState?.dockviewLayout;
    if (savedLayout) {
      try {
        dockviewApi.fromJSON(savedLayout);
        hasAnyPanels = (dockviewApi?.totalPanels ?? 0) > 0;
      } catch (err) {
        console.warn('Failed to restore dockview layout, starting fresh:', err);
      }
    }

    // Publish active file after restore
    tick().then(() => {
      if (dockviewApi?.activePanel) {
        const path = (dockviewApi.activePanel.params as EditorPanelParams | undefined)?.path ?? null;
        publishActiveFile(path);
      }

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

    // --- Bus subscriptions ---

    // Subscribe to compiler completion — trigger dirty state update in tabs
    unsubscribers.push(
      bus.subscribe('compiler.completed', () => {
        // Tab renderers have their own buffer subscriptions for dirty state
        // but we persist state in case file save changed dirty flags
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
    dockviewApi?.dispose();
    dockviewApi = undefined;
  });

  // --- State persistence ---

  function persistState(): void {
    if (!dockviewApi) return;
    try {
      const editorState: CodeEditorState = {
        dockviewLayout: dockviewApi.toJSON(),
      };
      onStateChange(editorState);
    } catch {
      // toJSON can fail during disposal — ignore
    }
  }

  function publishActiveFile(path: string | null): void {
    if (bus.hasChannel('editor.activeFile')) {
      bus.publish('editor.activeFile', { path });
    }
  }

  // --- Public API ---

  /** Open a file in the editor — creates or activates a dockview panel. */
  export function openFile(path: string, line?: number, col?: number): void {
    if (!dockviewApi) return;

    // Resolve compiler short filenames to full buffer paths
    const resolved = resolveCompilerPath(path, bufferMap.paths());
    path = resolved;

    const id = panelId(path);
    const existing = dockviewApi.getPanel(id);

    if (existing) {
      // Activate existing panel
      existing.api.setActive();
      // Scroll to line if requested
      if (line) {
        tick().then(() => {
          const renderer = getRenderer(id);
          renderer?.scrollToLine(line, col);
        });
      }
    } else {
      // Create new panel
      const params: EditorPanelParams = {
        path,
      };

      // Add to the active group if one exists, otherwise dockview creates one
      dockviewApi.addPanel({
        id,
        component: 'editorPanel',
        tabComponent: 'editorTab',
        title: path.split('/').pop() || path,
        params,
      });
    }

    persistState();
  }

  /** Close the active tab. */
  export function closeActiveTab(): void {
    if (!dockviewApi?.activePanel) return;
    dockviewApi.removePanel(dockviewApi.activePanel);
  }

  /** Switch to the next tab within the active group. */
  export function nextTab(): void {
    dockviewApi?.moveToNext({ includePanel: true });
  }

  /** Switch to the previous tab within the active group. */
  export function prevTab(): void {
    dockviewApi?.moveToPrevious({ includePanel: true });
  }

  /** Save the active panel's file to disk. */
  async function saveActiveFile(): Promise<void> {
    if (!dockviewApi?.activePanel || !fileSystem) return;
    const params = dockviewApi.activePanel.params as EditorPanelParams | undefined;
    const path = params?.path;
    if (!path) return;

    const content = bufferMap.get(path);
    if (content === undefined) return;

    try {
      await fileSystem.writeFile(path, content);
      bufferMap.markClean(path);
    } catch (err) {
      console.error('Failed to save file:', err);
    }
  }

  /** Get the EditorPane instance for the active panel (for external access). */
  export function getEditorPane(): unknown {
    // Not directly available — each panel owns its own EditorPane
    return undefined;
  }

  // Suppress unused prop warnings
  void zoneTypeId;
</script>

<div class="forge-code-editor-zone">
  <div class="forge-code-editor-zone__dockview" bind:this={containerEl}></div>

  {#if !hasAnyPanels}
    <div class="forge-code-editor-zone__empty">
      <p>No files open</p>
      <p class="forge-code-editor-zone__hint">Open a project to get started</p>
    </div>
  {/if}
</div>

<style>
  .forge-code-editor-zone {
    position: relative;
    width: 100%;
    height: 100%;
    overflow: hidden;
  }

  .forge-code-editor-zone__dockview {
    width: 100%;
    height: 100%;
  }

  .forge-code-editor-zone__empty {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    color: var(--forge-text-muted);
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-md);
    pointer-events: none;
  }

  .forge-code-editor-zone__hint {
    font-size: var(--forge-font-size-sm);
    margin-top: var(--forge-space-sm);
  }
</style>
