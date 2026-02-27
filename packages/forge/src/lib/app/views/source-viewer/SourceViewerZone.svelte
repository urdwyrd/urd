<script lang="ts">
  /**
   * SourceViewerZone — lightweight read-only file viewer.
   *
   * Single file, no tabs, no breadcrumb. Uses EditorPane with readOnly: true.
   * Supports multi-instance navigation (each zone views a different file).
   */

  import { onMount, onDestroy, tick } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { bufferMap } from '$lib/app/compiler/BufferMap';
  import { getCurrentTheme } from '$lib/framework/theme/ThemeEngine';
  import { appSettings } from '$lib/framework/settings/AppSettingsService';
  import { navigationBroker } from '$lib/framework/navigation/NavigationBroker';
  import EditorPane from '$lib/app/views/editor/EditorPane.svelte';

  interface SourceViewerState {
    filePath: string | null;
    scrollTop: number;
  }

  interface Props {
    zoneId: string;
    zoneTypeId: string;
    state: SourceViewerState | null;
    onStateChange: (newState: unknown) => void;
  }

  let { zoneId, zoneTypeId, state: zoneState, onStateChange }: Props = $props();

  let editorPane: EditorPane | undefined = $state();
  let currentTheme: 'gloaming' | 'parchment' = $state(getCurrentTheme());
  let filePath: string | null = $state(null);

  const unsubscribers: (() => void)[] = [];

  onMount(() => {
    if (zoneState) {
      filePath = zoneState.filePath ?? null;
    }

    tick().then(() => {
      // Handle navigation that arrived before this component mounted
      const pending = navigationBroker.consumePendingParams(zoneId);
      if (pending?.viewId === 'urd.sourceViewer' && pending.params.path) {
        openFile(pending.params.path as string, pending.params.line as number | undefined);
      } else {
        loadFile();
        // Restore scroll
        if (zoneState?.scrollTop && editorPane) {
          requestAnimationFrame(() => {
            editorPane?.setScrollTop(zoneState.scrollTop);
          });
        }
      }
    });

    unsubscribers.push(
      bus.subscribe('theme.changed', (payload) => {
        const { theme } = payload as { theme: string };
        currentTheme = theme as 'gloaming' | 'parchment';
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
        if (targetZoneId === zoneId && viewId === 'urd.sourceViewer' && params.path) {
          openFile(params.path as string, params.line as number | undefined);
        }
      })
    );

    // Refresh content on recompile (buffer may have changed)
    unsubscribers.push(
      bus.subscribe('compiler.completed', () => {
        loadFile();
      })
    );
  });

  onDestroy(() => {
    for (const unsub of unsubscribers) unsub();
    persistState();
  });

  function loadFile(): void {
    if (!editorPane || !filePath) return;
    const content = bufferMap.get(filePath);
    if (content !== undefined) {
      editorPane.setContent(content);
    }
  }

  function persistState(): void {
    const state: SourceViewerState = {
      filePath,
      scrollTop: editorPane?.getScrollTop() ?? 0,
    };
    onStateChange(state);
  }

  /** Open a file in the viewer — called externally via navigation. */
  export function openFile(path: string, line?: number): void {
    filePath = path;
    tick().then(() => {
      loadFile();
      if (line) {
        editorPane?.scrollToLine(line);
      }
    });
    persistState();
  }

  function fileName(path: string): string {
    return path.split('/').pop() || path;
  }

  // Settings-derived values
  let editorTabSize = $derived(appSettings.get('editorTabSize'));
  let editorLineNumbers = $derived(appSettings.get('editorLineNumbers'));
  let editorWordWrap = $derived(appSettings.get('editorWordWrap'));
</script>

<div class="forge-source-viewer-zone">
  {#if filePath}
    <div class="forge-source-viewer-zone__header">
      <span class="forge-source-viewer-zone__filename">{fileName(filePath)}</span>
      <span class="forge-source-viewer-zone__badge">Read-only</span>
    </div>
    <div class="forge-source-viewer-zone__editor">
      <EditorPane
        bind:this={editorPane}
        readOnly={true}
        theme={currentTheme}
        tabSize={editorTabSize}
        showLineNumbers={editorLineNumbers}
        wordWrap={editorWordWrap}
      />
    </div>
  {:else}
    <div class="forge-source-viewer-zone__empty">
      <p>No file selected</p>
      <p class="forge-source-viewer-zone__hint">Navigate to a file to view its source</p>
    </div>
  {/if}
</div>

<style>
  .forge-source-viewer-zone {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    overflow: hidden;
  }

  .forge-source-viewer-zone__header {
    display: flex;
    align-items: center;
    gap: var(--forge-space-sm);
    height: 28px;
    padding: 0 var(--forge-space-md);
    background-color: var(--forge-bg-secondary);
    border-bottom: 1px solid var(--forge-border-zone);
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-xs);
  }

  .forge-source-viewer-zone__filename {
    color: var(--forge-text-primary);
  }

  .forge-source-viewer-zone__badge {
    color: var(--forge-text-muted);
    font-size: 10px;
    padding: 0 4px;
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-sm);
  }

  .forge-source-viewer-zone__editor {
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }

  .forge-source-viewer-zone__empty {
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

  .forge-source-viewer-zone__hint {
    font-size: var(--forge-font-size-sm);
    margin-top: var(--forge-space-sm);
  }
</style>
