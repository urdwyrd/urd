<script lang="ts">
  /**
   * WhereUsed — shows all sites where a type or entity is referenced.
   */

  import { onMount, onDestroy } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { projectionRegistry } from '$lib/app/projections/ProjectionRegistry';
  import { selectionContext, type SelectionState } from '$lib/framework/selection/SelectionContext';
  import { navigationBroker } from '$lib/framework/navigation/NavigationBroker';
  import type { FactSet } from '$lib/app/compiler/types';

  interface Props { zoneId: string; zoneTypeId: string; state: null; onStateChange: (newState: unknown) => void; }
  let { zoneId, zoneTypeId, state: zoneState, onStateChange }: Props = $props();

  interface UsageResult { label: string; usageKind: string; file: string; line: number; }

  let selectedSymbol: string | null = $state(null);
  let results: UsageResult[] = $state([]);
  let selectedIndex: number = $state(0);
  const unsubscribers: (() => void)[] = [];

  onMount(() => {
    updateFromSelection(selectionContext.state);
    unsubscribers.push(selectionContext.subscribe(updateFromSelection));
    unsubscribers.push(bus.subscribe('compiler.completed', () => { if (selectedSymbol) findUsages(selectedSymbol); }));
  });
  onDestroy(() => { for (const unsub of unsubscribers) unsub(); });

  function updateFromSelection(state: SelectionState): void {
    if (state.items.length > 0) { selectedSymbol = state.items[0].id; findUsages(selectedSymbol); }
  }

  function findUsages(symbolId: string): void {
    const factSet = projectionRegistry.get<FactSet>('urd.projection.factSet');
    if (!factSet) { results = []; return; }
    const usages: UsageResult[] = [];
    for (const read of factSet.reads) {
      if (read.entity_type === symbolId) usages.push({ label: `${read.property} (read)`, usageKind: 'read', file: read.span.file, line: read.span.start_line });
    }
    for (const write of factSet.writes) {
      if (write.entity_type === symbolId) usages.push({ label: `${write.property} (write)`, usageKind: 'write', file: write.span.file, line: write.span.start_line });
    }
    for (const exit of factSet.exits) {
      if (exit.to_location === symbolId) usages.push({ label: `Exit from ${exit.from_location}`, usageKind: 'exit', file: exit.span.file, line: exit.span.start_line });
    }
    results = usages;
    selectedIndex = 0;
  }

  function navigateToResult(ref: UsageResult): void {
    navigationBroker.navigate({ targetViewId: 'urd.codeEditor', params: { path: ref.file, line: ref.line } });
  }
</script>

<div class="forge-where-used">
  <div class="forge-where-used__header">
    <span class="forge-where-used__title">Where Used</span>
    {#if selectedSymbol}<span class="forge-where-used__symbol">{selectedSymbol}</span>{/if}
  </div>
  <div class="forge-where-used__results">
    {#if !selectedSymbol}
      <div class="forge-where-used__empty">Select a symbol to see where it is used</div>
    {:else if results.length === 0}
      <div class="forge-where-used__empty">No usages found</div>
    {:else}
      {#each results as ref, i}
        <button class="forge-where-used__result" class:forge-where-used__result--selected={i === selectedIndex} onclick={() => navigateToResult(ref)} type="button">
          <span class="forge-where-used__badge forge-where-used__badge--{ref.usageKind}">{ref.usageKind}</span>
          <span class="forge-where-used__label">{ref.label}</span>
          <span class="forge-where-used__location">{ref.file}:{ref.line}</span>
        </button>
      {/each}
    {/if}
  </div>
</div>

<style>
  .forge-where-used { display: flex; flex-direction: column; width: 100%; height: 100%; overflow: hidden; font-family: var(--forge-font-family-ui); }
  .forge-where-used__header { display: flex; align-items: center; gap: var(--forge-space-sm); padding: var(--forge-space-md); background-color: var(--forge-bg-secondary); border-bottom: 1px solid var(--forge-border-zone); }
  .forge-where-used__title { font-size: var(--forge-font-size-xs); font-weight: 600; color: var(--forge-text-secondary); text-transform: uppercase; letter-spacing: 0.05em; }
  .forge-where-used__symbol { font-family: var(--forge-font-family-mono); font-size: var(--forge-font-size-xs); color: var(--forge-accent-primary); }
  .forge-where-used__results { flex: 1; min-height: 0; overflow-y: auto; }
  .forge-where-used__empty { display: flex; align-items: center; justify-content: center; padding: var(--forge-space-xl); color: var(--forge-text-muted); font-size: var(--forge-font-size-sm); }
  .forge-where-used__result { display: flex; align-items: center; gap: var(--forge-space-sm); width: 100%; height: 28px; padding: 0 var(--forge-space-md); background: none; border: none; cursor: pointer; font-family: var(--forge-font-family-ui); font-size: var(--forge-font-size-sm); color: var(--forge-text-primary); text-align: left; }
  .forge-where-used__result:hover { background-color: var(--forge-table-row-hover); }
  .forge-where-used__result--selected { background-color: var(--forge-accent-selection); }
  .forge-where-used__badge { padding: 0 5px; border-radius: var(--forge-radius-sm); font-size: 10px; font-weight: 600; text-transform: uppercase; line-height: 18px; flex-shrink: 0; }
  .forge-where-used__badge--read { background: var(--forge-runtime-event-move, #5b9bd5); color: #fff; }
  .forge-where-used__badge--write { background: var(--forge-runtime-event-set, #4caf50); color: #fff; }
  .forge-where-used__badge--exit { background: var(--forge-runtime-event-narration, #888); color: #fff; }
  .forge-where-used__label { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .forge-where-used__location { flex-shrink: 0; color: var(--forge-text-muted); font-family: var(--forge-font-family-mono); font-size: var(--forge-font-size-xs); }
</style>
