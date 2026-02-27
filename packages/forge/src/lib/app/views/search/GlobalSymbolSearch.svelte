<script lang="ts">
  /**
   * GlobalSymbolSearch — fuzzy substring search over definitionIndex projection.
   *
   * Text input with results list. Arrow keys for selection, Enter/click navigates
   * to definition.
   */

  import { onMount, onDestroy, tick } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { projectionRegistry } from '$lib/app/projections/ProjectionRegistry';
  import { navigationBroker } from '$lib/framework/navigation/NavigationBroker';
  import type { DefinitionIndex, DefinitionEntry } from '$lib/app/projections/definition-index';

  interface Props {
    zoneId: string;
    zoneTypeId: string;
    state: null;
    onStateChange: (newState: unknown) => void;
  }

  let { zoneId, zoneTypeId, state: zoneState, onStateChange }: Props = $props();

  let query: string = $state('');
  let allDefs: DefinitionEntry[] = $state([]);
  let selectedIndex: number = $state(0);
  let inputEl: HTMLInputElement | undefined = $state();

  let results = $derived.by(() => {
    if (!query) return allDefs.slice(0, 50);
    const lower = query.toLowerCase();
    return allDefs
      .filter((d) => {
        const name = (d.definition.display_name ?? d.key).toLowerCase();
        return name.includes(lower);
      })
      .slice(0, 50);
  });

  const unsubscribers: (() => void)[] = [];

  onMount(() => {
    refreshData();
    unsubscribers.push(bus.subscribe('compiler.completed', refreshData));
    // Focus the input on mount
    tick().then(() => inputEl?.focus());
  });

  onDestroy(() => {
    for (const unsub of unsubscribers) unsub();
  });

  function refreshData(): void {
    allDefs = projectionRegistry.get<DefinitionIndex>('urd.projection.definitionIndex') ?? [];
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

  function navigateToResult(entry: DefinitionEntry): void {
    navigationBroker.navigate({
      targetViewId: 'urd.codeEditor',
      params: { path: entry.span.file, line: entry.span.start_line },
    });
  }

  function handleInput(e: Event): void {
    query = (e.target as HTMLInputElement).value;
    selectedIndex = 0;
  }

  function handleKeydown(e: KeyboardEvent): void {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      selectedIndex = Math.min(selectedIndex + 1, results.length - 1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      selectedIndex = Math.max(selectedIndex - 1, 0);
    } else if (e.key === 'Enter') {
      e.preventDefault();
      if (results[selectedIndex]) {
        navigateToResult(results[selectedIndex]);
      }
    }
  }
</script>

<div class="forge-global-search">
  <div class="forge-global-search__header">
    <input
      bind:this={inputEl}
      class="forge-global-search__input"
      type="text"
      value={query}
      placeholder="Search symbols\u2026"
      oninput={handleInput}
      onkeydown={handleKeydown}
      spellcheck="false"
      autocomplete="off"
    />
  </div>

  <div class="forge-global-search__results">
    {#if results.length === 0}
      <div class="forge-global-search__empty">
        {query ? 'No matching symbols' : 'No symbols available'}
      </div>
    {:else}
      {#each results as entry, i (entry.key)}
        <button
          class="forge-global-search__result"
          class:forge-global-search__result--selected={i === selectedIndex}
          role="option"
          aria-selected={i === selectedIndex}
          onclick={() => navigateToResult(entry)}
          tabindex="-1"
          type="button"
        >
          <span class="forge-global-search__kind forge-global-search__kind--{entry.definition.kind}">
            {kindIcon(entry.definition.kind)}
          </span>
          <span class="forge-global-search__name">{entry.definition.display_name ?? entry.key}</span>
          <span class="forge-global-search__location">
            {entry.span.file}:{entry.span.start_line}
          </span>
        </button>
      {/each}
    {/if}
  </div>
</div>

<style>
  .forge-global-search {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    overflow: hidden;
    font-family: var(--forge-font-family-ui);
  }

  .forge-global-search__header {
    padding: var(--forge-space-md);
    background-color: var(--forge-bg-secondary);
    border-bottom: 1px solid var(--forge-border-zone);
  }

  .forge-global-search__input {
    width: 100%;
    height: 26px;
    padding: 0 var(--forge-space-md);
    background-color: var(--forge-bg-tertiary);
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-sm);
    color: var(--forge-text-primary);
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-sm);
    outline: none;
  }

  .forge-global-search__input:focus {
    border-color: var(--forge-accent-primary);
  }

  .forge-global-search__input::placeholder {
    color: var(--forge-text-muted);
  }

  .forge-global-search__results {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
  }

  .forge-global-search__empty {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--forge-space-xl);
    color: var(--forge-text-muted);
    font-size: var(--forge-font-size-sm);
  }

  .forge-global-search__result {
    display: flex;
    align-items: center;
    gap: var(--forge-space-sm);
    width: 100%;
    height: 28px;
    padding: 0 var(--forge-space-md);
    background: none;
    border: none;
    cursor: pointer;
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-sm);
    color: var(--forge-text-primary);
    text-align: left;
  }

  .forge-global-search__result:hover {
    background-color: var(--forge-table-row-hover);
  }

  .forge-global-search__result--selected {
    background-color: var(--forge-accent-selection);
  }

  .forge-global-search__kind {
    flex-shrink: 0;
    width: 14px;
    text-align: center;
    font-size: var(--forge-font-size-xs);
  }

  .forge-global-search__kind--entity { color: var(--forge-syntax-entity); }
  .forge-global-search__kind--location { color: var(--forge-syntax-string); }
  .forge-global-search__kind--section { color: var(--forge-syntax-heading); }
  .forge-global-search__kind--type { color: var(--forge-syntax-keyword); }
  .forge-global-search__kind--property { color: var(--forge-syntax-property); }

  .forge-global-search__name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .forge-global-search__location {
    flex-shrink: 0;
    color: var(--forge-text-muted);
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-xs);
  }
</style>
