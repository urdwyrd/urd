<script lang="ts">
  /**
   * ConditionPredicateSearch — find rules and choices that read specific properties.
   */

  import { onMount, onDestroy, tick } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { projectionRegistry } from '$lib/app/projections/ProjectionRegistry';
  import { navigationBroker } from '$lib/framework/navigation/NavigationBroker';
  import type { FactSet } from '$lib/app/compiler/types';

  interface Props { zoneId: string; zoneTypeId: string; state: null; onStateChange: (newState: unknown) => void; }
  let { zoneId, zoneTypeId, state: zoneState, onStateChange }: Props = $props();

  interface CondResult { siteKind: string; siteId: string; entityType: string; property: string; operator: string; value: string; file: string; line: number; }

  let query: string = $state('');
  let results: CondResult[] = $state([]);
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
    const factSet = projectionRegistry.get<FactSet>('urd.projection.factSet');
    if (!factSet) { results = []; return; }
    const lower = query.toLowerCase();
    const matches: CondResult[] = [];

    for (const read of factSet.reads) {
      if (read.entity_type.toLowerCase().includes(lower) || read.property.toLowerCase().includes(lower)) {
        matches.push({ siteKind: read.site.kind, siteId: read.site.id, entityType: read.entity_type, property: read.property, operator: read.operator, value: read.value_literal, file: read.span.file, line: read.span.start_line });
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

  function navigateToResult(ref: CondResult): void {
    navigationBroker.navigate({ targetViewId: 'urd.codeEditor', params: { path: ref.file, line: ref.line } });
  }

  function handleKeydown(e: KeyboardEvent): void {
    if (e.key === 'ArrowDown') { e.preventDefault(); selectedIndex = Math.min(selectedIndex + 1, results.length - 1); }
    else if (e.key === 'ArrowUp') { e.preventDefault(); selectedIndex = Math.max(selectedIndex - 1, 0); }
    else if (e.key === 'Enter' && results[selectedIndex]) { e.preventDefault(); navigateToResult(results[selectedIndex]); }
  }
</script>

<div class="forge-cond-search">
  <div class="forge-cond-search__header">
    <input bind:this={inputEl} class="forge-cond-search__input" type="text" value={query} placeholder="Search conditions (e.g. Player.health)\u2026" oninput={handleInput} onkeydown={handleKeydown} spellcheck="false" autocomplete="off" />
  </div>
  <div class="forge-cond-search__results">
    {#if results.length === 0}
      <div class="forge-cond-search__empty">{query ? 'No matching conditions' : 'Search for entity type or property name'}</div>
    {:else}
      {#each results as ref, i}
        <button class="forge-cond-search__result" class:forge-cond-search__result--selected={i === selectedIndex} onclick={() => navigateToResult(ref)} type="button">
          <span class="forge-cond-search__site">{ref.siteKind}:{ref.siteId}</span>
          <span class="forge-cond-search__condition">{ref.entityType}.{ref.property} {ref.operator} {ref.value}</span>
          <span class="forge-cond-search__location">{ref.file}:{ref.line}</span>
        </button>
      {/each}
    {/if}
  </div>
</div>

<style>
  .forge-cond-search { display: flex; flex-direction: column; width: 100%; height: 100%; overflow: hidden; font-family: var(--forge-font-family-ui); }
  .forge-cond-search__header { padding: var(--forge-space-md); background-color: var(--forge-bg-secondary); border-bottom: 1px solid var(--forge-border-zone); }
  .forge-cond-search__input { width: 100%; height: 26px; padding: 0 var(--forge-space-md); background-color: var(--forge-bg-tertiary); border: 1px solid var(--forge-border-zone); border-radius: var(--forge-radius-sm); color: var(--forge-text-primary); font-family: var(--forge-font-family-ui); font-size: var(--forge-font-size-sm); outline: none; }
  .forge-cond-search__input:focus { border-color: var(--forge-accent-primary); }
  .forge-cond-search__results { flex: 1; min-height: 0; overflow-y: auto; }
  .forge-cond-search__empty { display: flex; align-items: center; justify-content: center; padding: var(--forge-space-xl); color: var(--forge-text-muted); font-size: var(--forge-font-size-sm); }
  .forge-cond-search__result { display: flex; align-items: center; gap: var(--forge-space-sm); width: 100%; height: 28px; padding: 0 var(--forge-space-md); background: none; border: none; cursor: pointer; font-family: var(--forge-font-family-ui); font-size: var(--forge-font-size-sm); color: var(--forge-text-primary); text-align: left; }
  .forge-cond-search__result:hover { background-color: var(--forge-table-row-hover); }
  .forge-cond-search__result--selected { background-color: var(--forge-accent-selection); }
  .forge-cond-search__site { flex-shrink: 0; color: var(--forge-text-muted); font-size: var(--forge-font-size-xs); min-width: 100px; }
  .forge-cond-search__condition { flex: 1; font-family: var(--forge-font-family-mono); font-size: var(--forge-font-size-xs); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .forge-cond-search__location { flex-shrink: 0; color: var(--forge-text-muted); font-family: var(--forge-font-family-mono); font-size: var(--forge-font-size-xs); }
</style>
