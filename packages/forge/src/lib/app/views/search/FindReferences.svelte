<script lang="ts">
  /**
   * FindReferences — shows all references to the currently selected symbol.
   */

  import { onMount, onDestroy, tick } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { projectionRegistry } from '$lib/app/projections/ProjectionRegistry';
  import { selectionContext, type SelectionState } from '$lib/framework/selection/SelectionContext';
  import { navigationBroker } from '$lib/framework/navigation/NavigationBroker';
  import type { FactSet } from '$lib/app/compiler/types';

  interface Props {
    zoneId: string;
    zoneTypeId: string;
    state: null;
    onStateChange: (newState: unknown) => void;
  }

  let { zoneId, zoneTypeId, state: zoneState, onStateChange }: Props = $props();

  interface RefResult { label: string; context: string; file: string; line: number; }

  let selectedSymbol: string | null = $state(null);
  let results: RefResult[] = $state([]);
  let selectedIndex: number = $state(0);

  const unsubscribers: (() => void)[] = [];

  onMount(() => {
    updateFromSelection(selectionContext.state);
    unsubscribers.push(selectionContext.subscribe(updateFromSelection));
    unsubscribers.push(bus.subscribe('compiler.completed', () => {
      if (selectedSymbol) findRefs(selectedSymbol);
    }));
  });

  onDestroy(() => {
    for (const unsub of unsubscribers) unsub();
  });

  function updateFromSelection(state: SelectionState): void {
    if (state.items.length > 0) {
      selectedSymbol = state.items[0].id;
      findRefs(selectedSymbol);
    }
  }

  function findRefs(symbolId: string): void {
    const factSet = projectionRegistry.get<FactSet>('urd.projection.factSet');
    if (!factSet) { results = []; return; }
    const refs: RefResult[] = [];
    const lower = symbolId.toLowerCase();

    for (const read of factSet.reads) {
      if (read.entity_type.toLowerCase().includes(lower) || read.property.toLowerCase().includes(lower)) {
        refs.push({ label: `Read: ${read.entity_type}.${read.property}`, context: `${read.site.kind}:${read.site.id}`, file: read.span.file, line: read.span.start_line });
      }
    }
    for (const write of factSet.writes) {
      if (write.entity_type.toLowerCase().includes(lower) || write.property.toLowerCase().includes(lower)) {
        refs.push({ label: `Write: ${write.entity_type}.${write.property}`, context: `${write.site.kind}:${write.site.id}`, file: write.span.file, line: write.span.start_line });
      }
    }
    for (const exit of factSet.exits) {
      if (exit.from_location.toLowerCase().includes(lower) || exit.to_location.toLowerCase().includes(lower)) {
        refs.push({ label: `Exit: ${exit.from_location} → ${exit.to_location}`, context: exit.exit_name, file: exit.span.file, line: exit.span.start_line });
      }
    }
    results = refs;
    selectedIndex = 0;
  }

  function navigateToResult(ref: RefResult): void {
    navigationBroker.navigate({ targetViewId: 'urd.codeEditor', params: { path: ref.file, line: ref.line } });
  }

  function handleKeydown(e: KeyboardEvent): void {
    if (e.key === 'ArrowDown') { e.preventDefault(); selectedIndex = Math.min(selectedIndex + 1, results.length - 1); }
    else if (e.key === 'ArrowUp') { e.preventDefault(); selectedIndex = Math.max(selectedIndex - 1, 0); }
    else if (e.key === 'Enter' && results[selectedIndex]) { e.preventDefault(); navigateToResult(results[selectedIndex]); }
  }
</script>

<div class="forge-find-refs" onkeydown={handleKeydown}>
  <div class="forge-find-refs__header">
    <span class="forge-find-refs__title">Find References</span>
    {#if selectedSymbol}
      <span class="forge-find-refs__symbol">{selectedSymbol}</span>
    {/if}
  </div>
  <div class="forge-find-refs__results">
    {#if !selectedSymbol}
      <div class="forge-find-refs__empty">Select a symbol to find references</div>
    {:else if results.length === 0}
      <div class="forge-find-refs__empty">No references found</div>
    {:else}
      {#each results as ref, i}
        <button class="forge-find-refs__result" class:forge-find-refs__result--selected={i === selectedIndex} onclick={() => navigateToResult(ref)} type="button">
          <span class="forge-find-refs__label">{ref.label}</span>
          <span class="forge-find-refs__context">{ref.context}</span>
          <span class="forge-find-refs__location">{ref.file}:{ref.line}</span>
        </button>
      {/each}
    {/if}
  </div>
</div>

<style>
  .forge-find-refs { display: flex; flex-direction: column; width: 100%; height: 100%; overflow: hidden; font-family: var(--forge-font-family-ui); }
  .forge-find-refs__header { display: flex; align-items: center; gap: var(--forge-space-sm); padding: var(--forge-space-md); background-color: var(--forge-bg-secondary); border-bottom: 1px solid var(--forge-border-zone); }
  .forge-find-refs__title { font-size: var(--forge-font-size-xs); font-weight: 600; color: var(--forge-text-secondary); text-transform: uppercase; letter-spacing: 0.05em; }
  .forge-find-refs__symbol { font-family: var(--forge-font-family-mono); font-size: var(--forge-font-size-xs); color: var(--forge-accent-primary); }
  .forge-find-refs__results { flex: 1; min-height: 0; overflow-y: auto; }
  .forge-find-refs__empty { display: flex; align-items: center; justify-content: center; padding: var(--forge-space-xl); color: var(--forge-text-muted); font-size: var(--forge-font-size-sm); }
  .forge-find-refs__result { display: flex; align-items: center; gap: var(--forge-space-sm); width: 100%; height: 28px; padding: 0 var(--forge-space-md); background: none; border: none; cursor: pointer; font-family: var(--forge-font-family-ui); font-size: var(--forge-font-size-sm); color: var(--forge-text-primary); text-align: left; }
  .forge-find-refs__result:hover { background-color: var(--forge-table-row-hover); }
  .forge-find-refs__result--selected { background-color: var(--forge-accent-selection); }
  .forge-find-refs__label { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .forge-find-refs__context { color: var(--forge-text-muted); font-size: var(--forge-font-size-xs); }
  .forge-find-refs__location { flex-shrink: 0; color: var(--forge-text-muted); font-family: var(--forge-font-family-mono); font-size: var(--forge-font-size-xs); }
</style>
