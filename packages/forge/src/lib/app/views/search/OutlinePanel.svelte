<script lang="ts">
  /**
   * OutlinePanel — symbol tree for the active editor file.
   *
   * Listens to editor.activeFile bus channel and filters the outline
   * projection to show only symbols from the current file.
   */

  import { onMount, onDestroy } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { projectionRegistry } from '$lib/app/projections/ProjectionRegistry';
  import { navigationBroker } from '$lib/framework/navigation/NavigationBroker';
  import type { FileOutline, OutlineEntry } from '$lib/app/projections/outline';

  interface Props {
    zoneId: string;
    zoneTypeId: string;
    state: null;
    onStateChange: (newState: unknown) => void;
  }

  let { zoneId, zoneTypeId, state: zoneState, onStateChange }: Props = $props();

  let activeFile: string | null = $state(null);
  let allOutlines: FileOutline[] = $state([]);
  let selectedEntryId: string | null = $state(null);

  let entries = $derived.by(() => {
    if (!activeFile) return [];
    const outline = allOutlines.find((o) => o.file === activeFile);
    return outline?.entries ?? [];
  });

  const unsubscribers: (() => void)[] = [];

  onMount(() => {
    refreshData();

    // Listen for active file changes
    unsubscribers.push(
      bus.subscribe('editor.activeFile', (payload) => {
        const { path } = payload as { path: string | null };
        activeFile = path;
      })
    );

    // Also read the last published value
    const lastActive = bus.getLastValue('editor.activeFile') as { path: string | null } | undefined;
    if (lastActive?.path) {
      activeFile = lastActive.path;
    }

    unsubscribers.push(bus.subscribe('compiler.completed', refreshData));
  });

  onDestroy(() => {
    for (const unsub of unsubscribers) unsub();
  });

  function refreshData(): void {
    allOutlines = projectionRegistry.get<FileOutline[]>('urd.projection.outline') ?? [];
  }

  function kindIcon(kind: string): string {
    switch (kind) {
      case 'entity': return '\u25C6';
      case 'location': return '\u25CB';
      case 'section': return '\u00A7';
      case 'type': return '\u25A0';
      case 'property': return '\u25AA';
      default: return '\u25C7';
    }
  }

  function goToSymbol(entry: OutlineEntry): void {
    selectedEntryId = entry.id;
    navigationBroker.navigate({
      targetViewId: 'urd.codeEditor',
      params: { path: entry.file, line: entry.line },
    });
  }

  function handleKeydown(e: KeyboardEvent, entry: OutlineEntry): void {
    if (e.key === 'Enter') {
      goToSymbol(entry);
    }
  }
</script>

<div class="forge-outline-panel">
  {#if !activeFile}
    <div class="forge-outline-panel__empty">
      No file open
    </div>
  {:else if entries.length === 0}
    <div class="forge-outline-panel__empty">
      No symbols in this file
    </div>
  {:else}
    <div class="forge-outline-panel__list">
      {#each entries as entry (entry.id)}
        <div
          class="forge-outline-panel__entry"
          class:forge-outline-panel__entry--selected={selectedEntryId === entry.id}
          role="button"
          tabindex="0"
          onclick={() => goToSymbol(entry)}
          onkeydown={(e) => handleKeydown(e, entry)}
        >
          <span class="forge-outline-panel__icon forge-outline-panel__icon--{entry.kind}">{kindIcon(entry.kind)}</span>
          <span class="forge-outline-panel__name">{entry.name}</span>
          <span class="forge-outline-panel__line">{entry.line}</span>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .forge-outline-panel {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    overflow-y: auto;
    font-family: var(--forge-font-family-ui);
  }

  .forge-outline-panel__empty {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    color: var(--forge-text-muted);
    font-size: var(--forge-font-size-md);
  }

  .forge-outline-panel__list {
    padding: var(--forge-space-xs) 0;
  }

  .forge-outline-panel__entry {
    display: flex;
    align-items: center;
    gap: var(--forge-space-sm);
    height: 24px;
    padding: 0 var(--forge-space-md);
    cursor: pointer;
    font-size: var(--forge-font-size-sm);
    color: var(--forge-text-primary);
  }

  .forge-outline-panel__entry:hover {
    background-color: var(--forge-table-row-hover);
  }

  .forge-outline-panel__entry--selected {
    background-color: var(--forge-accent-selection);
  }

  .forge-outline-panel__entry:focus-visible {
    outline: 1px solid var(--forge-accent-primary);
    outline-offset: -1px;
  }

  .forge-outline-panel__icon {
    flex-shrink: 0;
    width: 14px;
    text-align: center;
    font-size: var(--forge-font-size-xs);
  }

  .forge-outline-panel__icon--entity { color: var(--forge-syntax-entity); }
  .forge-outline-panel__icon--location { color: var(--forge-syntax-string); }
  .forge-outline-panel__icon--section { color: var(--forge-syntax-heading); }
  .forge-outline-panel__icon--type { color: var(--forge-syntax-keyword); }
  .forge-outline-panel__icon--property { color: var(--forge-syntax-property); }

  .forge-outline-panel__name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .forge-outline-panel__line {
    flex-shrink: 0;
    color: var(--forge-text-muted);
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-xs);
  }
</style>
