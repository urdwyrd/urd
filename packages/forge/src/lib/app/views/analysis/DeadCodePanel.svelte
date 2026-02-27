<script lang="ts">
  /**
   * DeadCodePanel — lists unreferenced symbols from the deadCode projection.
   */

  import { onMount, onDestroy } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { projectionRegistry } from '$lib/app/projections/ProjectionRegistry';
  import { navigationBroker } from '$lib/framework/navigation/NavigationBroker';
  import type { DeadCodeEntry } from '$lib/app/projections/dead-code';

  interface Props {
    zoneId: string;
    zoneTypeId: string;
    state: null;
    onStateChange: (newState: unknown) => void;
  }

  let { zoneId, zoneTypeId, state: zoneState, onStateChange }: Props = $props();

  let entries: DeadCodeEntry[] = $state([]);

  const unsubscribers: (() => void)[] = [];

  onMount(() => {
    refreshData();
    unsubscribers.push(bus.subscribe('compiler.completed', refreshData));
  });

  onDestroy(() => {
    for (const unsub of unsubscribers) unsub();
  });

  function refreshData(): void {
    entries = projectionRegistry.get<DeadCodeEntry[]>('urd.projection.deadCode') ?? [];
  }

  // Group by kind
  let grouped = $derived.by(() => {
    const groups = new Map<string, DeadCodeEntry[]>();
    for (const entry of entries) {
      const list = groups.get(entry.kind) ?? [];
      list.push(entry);
      groups.set(entry.kind, list);
    }
    return [...groups.entries()].sort((a, b) => a[0].localeCompare(b[0]));
  });

  function goToSource(entry: DeadCodeEntry): void {
    if (entry.file) {
      navigationBroker.navigate({
        targetViewId: 'urd.codeEditor',
        params: { path: entry.file, line: entry.line },
      });
    }
  }
</script>

<div class="forge-dead-code-panel">
  {#if entries.length === 0}
    <div class="forge-dead-code-panel__empty">
      No dead code detected
    </div>
  {:else}
    <div class="forge-dead-code-panel__list">
      {#each grouped as [kind, items]}
        <div class="forge-dead-code-panel__group">
          <div class="forge-dead-code-panel__group-header">
            {kind} ({items.length})
          </div>
          {#each items as entry}
            <button
              class="forge-dead-code-panel__entry"
              onclick={() => goToSource(entry)}
            >
              <span class="forge-dead-code-panel__name">{entry.name}</span>
              <span class="forge-dead-code-panel__location">{entry.file}:{entry.line}</span>
            </button>
          {/each}
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .forge-dead-code-panel {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    overflow-y: auto;
    font-family: var(--forge-font-family-ui);
  }

  .forge-dead-code-panel__empty {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    color: var(--forge-text-muted);
    font-size: var(--forge-font-size-md);
  }

  .forge-dead-code-panel__list {
    padding: var(--forge-space-md);
  }

  .forge-dead-code-panel__group {
    margin-bottom: var(--forge-space-md);
  }

  .forge-dead-code-panel__group-header {
    font-size: var(--forge-font-size-xs);
    font-weight: 600;
    color: var(--forge-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    padding: var(--forge-space-xs) 0;
    border-bottom: 1px solid var(--forge-border-zone);
    margin-bottom: var(--forge-space-xs);
  }

  .forge-dead-code-panel__entry {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: var(--forge-space-xs) var(--forge-space-sm);
    background: none;
    border: none;
    border-radius: var(--forge-radius-sm);
    cursor: pointer;
    text-align: left;
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-sm);
    color: var(--forge-text-primary);
  }

  .forge-dead-code-panel__entry:hover {
    background-color: var(--forge-table-row-hover);
  }

  .forge-dead-code-panel__name {
    font-weight: 500;
  }

  .forge-dead-code-panel__location {
    color: var(--forge-text-muted);
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-xs);
  }
</style>
