<script lang="ts">
  /**
   * PropertyValueSearch — find entities with specific property values.
   */

  import { onMount, onDestroy, tick } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { projectionRegistry } from '$lib/app/projections/ProjectionRegistry';
  import { selectionContext } from '$lib/framework/selection/SelectionContext';
  import { navigationBroker } from '$lib/framework/navigation/NavigationBroker';
  import type { UrdWorld, SymbolTable } from '$lib/app/compiler/types';

  interface Props { zoneId: string; zoneTypeId: string; state: null; onStateChange: (newState: unknown) => void; }
  let { zoneId, zoneTypeId, state: zoneState, onStateChange }: Props = $props();

  interface SearchResult { entityId: string; entityName: string; property: string; value: string; file: string; line: number; }

  let query: string = $state('');
  let results: SearchResult[] = $state([]);
  let selectedIndex: number = $state(0);
  let inputEl: HTMLInputElement | undefined = $state();
  const unsubscribers: (() => void)[] = [];

  onMount(() => {
    unsubscribers.push(bus.subscribe('compiler.completed', () => { if (query) runSearch(); }));
    tick().then(() => inputEl?.focus());
  });
  onDestroy(() => { for (const unsub of unsubscribers) unsub(); });

  function runSearch(): void {
    if (!query) { results = []; return; }
    const urdJson = projectionRegistry.get<UrdWorld>('urd.projection.urdJson');
    const symTable = projectionRegistry.get<SymbolTable>('urd.projection.symbolTable');
    if (!urdJson) { results = []; return; }
    const lower = query.toLowerCase();
    const matches: SearchResult[] = [];

    for (const entity of urdJson.entities) {
      for (const [key, value] of Object.entries(entity.properties)) {
        const valStr = String(value ?? '');
        if (key.toLowerCase().includes(lower) || valStr.toLowerCase().includes(lower)) {
          const sym = symTable?.entries.find((e) => e.id === entity.id);
          matches.push({ entityId: entity.id, entityName: entity.name, property: key, value: valStr, file: sym?.file ?? '', line: sym?.line ?? 0 });
        }
        if (matches.length >= 100) break;
      }
      if (matches.length >= 100) break;
    }
    results = matches;
    selectedIndex = 0;
  }

  function handleInput(e: Event): void {
    query = (e.target as HTMLInputElement).value;
    runSearch();
  }

  function handleRowClick(ref: SearchResult, index: number): void {
    selectedIndex = index;
    selectionContext.select([{ kind: 'entity', id: ref.entityId, label: ref.entityName, data: { file: ref.file, line: ref.line } }], zoneId);
  }

  function handleRowDoubleClick(ref: SearchResult): void {
    if (ref.file) navigationBroker.navigate({ targetViewId: 'urd.codeEditor', params: { path: ref.file, line: ref.line } });
  }

  function handleKeydown(e: KeyboardEvent): void {
    if (e.key === 'ArrowDown') { e.preventDefault(); selectedIndex = Math.min(selectedIndex + 1, results.length - 1); }
    else if (e.key === 'ArrowUp') { e.preventDefault(); selectedIndex = Math.max(selectedIndex - 1, 0); }
    else if (e.key === 'Enter' && results[selectedIndex]) { e.preventDefault(); handleRowDoubleClick(results[selectedIndex]); }
  }
</script>

<div class="forge-prop-search">
  <div class="forge-prop-search__header">
    <input bind:this={inputEl} class="forge-prop-search__input" type="text" value={query} placeholder="Search property values\u2026" oninput={handleInput} onkeydown={handleKeydown} spellcheck="false" autocomplete="off" />
  </div>
  <div class="forge-prop-search__results">
    {#if results.length === 0}
      <div class="forge-prop-search__empty">{query ? 'No matching properties' : 'Search for property names or values'}</div>
    {:else}
      {#each results as ref, i}
        <button class="forge-prop-search__result" class:forge-prop-search__result--selected={i === selectedIndex} onclick={() => handleRowClick(ref, i)} ondblclick={() => handleRowDoubleClick(ref)} type="button">
          <span class="forge-prop-search__entity">{ref.entityName}</span>
          <span class="forge-prop-search__prop">{ref.property}</span>
          <span class="forge-prop-search__value">{ref.value}</span>
        </button>
      {/each}
    {/if}
  </div>
</div>

<style>
  .forge-prop-search { display: flex; flex-direction: column; width: 100%; height: 100%; overflow: hidden; font-family: var(--forge-font-family-ui); }
  .forge-prop-search__header { padding: var(--forge-space-md); background-color: var(--forge-bg-secondary); border-bottom: 1px solid var(--forge-border-zone); }
  .forge-prop-search__input { width: 100%; height: 26px; padding: 0 var(--forge-space-md); background-color: var(--forge-bg-tertiary); border: 1px solid var(--forge-border-zone); border-radius: var(--forge-radius-sm); color: var(--forge-text-primary); font-family: var(--forge-font-family-ui); font-size: var(--forge-font-size-sm); outline: none; }
  .forge-prop-search__input:focus { border-color: var(--forge-accent-primary); }
  .forge-prop-search__results { flex: 1; min-height: 0; overflow-y: auto; }
  .forge-prop-search__empty { display: flex; align-items: center; justify-content: center; padding: var(--forge-space-xl); color: var(--forge-text-muted); font-size: var(--forge-font-size-sm); }
  .forge-prop-search__result { display: flex; align-items: center; gap: var(--forge-space-sm); width: 100%; height: 28px; padding: 0 var(--forge-space-md); background: none; border: none; cursor: pointer; font-family: var(--forge-font-family-ui); font-size: var(--forge-font-size-sm); color: var(--forge-text-primary); text-align: left; }
  .forge-prop-search__result:hover { background-color: var(--forge-table-row-hover); }
  .forge-prop-search__result--selected { background-color: var(--forge-accent-selection); }
  .forge-prop-search__entity { font-weight: 600; min-width: 100px; }
  .forge-prop-search__prop { color: var(--forge-text-secondary); font-family: var(--forge-font-family-mono); font-size: var(--forge-font-size-xs); min-width: 80px; }
  .forge-prop-search__value { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: var(--forge-text-muted); font-size: var(--forge-font-size-xs); }
</style>
