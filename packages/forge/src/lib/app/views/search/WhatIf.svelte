<script lang="ts">
  /**
   * WhatIf — hypothetical impact analysis. Shows what would break if a symbol were removed.
   */

  import { onMount, onDestroy } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { projectionRegistry } from '$lib/app/projections/ProjectionRegistry';
  import { selectionContext, type SelectionState } from '$lib/framework/selection/SelectionContext';
  import { navigationBroker } from '$lib/framework/navigation/NavigationBroker';
  import type { FactSet } from '$lib/app/compiler/types';

  interface Props { zoneId: string; zoneTypeId: string; state: null; onStateChange: (newState: unknown) => void; }
  let { zoneId, zoneTypeId, state: zoneState, onStateChange }: Props = $props();

  interface ImpactItem { description: string; severity: 'high' | 'medium' | 'low'; file: string; line: number; }

  let selectedSymbol: string | null = $state(null);
  let impacts: ImpactItem[] = $state([]);
  const unsubscribers: (() => void)[] = [];

  onMount(() => {
    updateFromSelection(selectionContext.state);
    unsubscribers.push(selectionContext.subscribe(updateFromSelection));
    unsubscribers.push(bus.subscribe('compiler.completed', () => { if (selectedSymbol) computeImpact(selectedSymbol); }));
  });
  onDestroy(() => { for (const unsub of unsubscribers) unsub(); });

  function updateFromSelection(state: SelectionState): void {
    if (state.items.length > 0) { selectedSymbol = state.items[0].id; computeImpact(selectedSymbol); }
  }

  function computeImpact(symbolId: string): void {
    const factSet = projectionRegistry.get<FactSet>('urd.projection.factSet');
    if (!factSet) { impacts = []; return; }
    const items: ImpactItem[] = [];

    // Check rules referencing this symbol
    for (const rule of factSet.rules) {
      const condReads = rule.condition_reads.map((i) => factSet.reads[i]).filter(Boolean);
      const effWrites = rule.effect_writes.map((i) => factSet.writes[i]).filter(Boolean);
      const references = [...condReads, ...effWrites].some((rw) => rw.entity_type === symbolId || rw.property === symbolId);
      if (references) items.push({ description: `Rule "${rule.rule_id}" would be broken`, severity: 'high', file: rule.span.file, line: rule.span.start_line });
    }

    // Check choices referencing this symbol
    for (const choice of factSet.choices) {
      const condReads = choice.condition_reads.map((i) => factSet.reads[i]).filter(Boolean);
      const references = condReads.some((r) => r.entity_type === symbolId || r.property === symbolId);
      if (references) items.push({ description: `Choice "${choice.label}" condition would be invalid`, severity: 'medium', file: choice.span.file, line: choice.span.start_line });
    }

    // Check exits referencing this as a location
    for (const exit of factSet.exits) {
      if (exit.to_location === symbolId) items.push({ description: `Exit from "${exit.from_location}" would have no target`, severity: 'high', file: exit.span.file, line: exit.span.start_line });
      if (exit.from_location === symbolId) items.push({ description: `Exit to "${exit.to_location}" would be orphaned`, severity: 'medium', file: exit.span.file, line: exit.span.start_line });
    }

    impacts = items;
  }

  function navigateToItem(item: ImpactItem): void {
    navigationBroker.navigate({ targetViewId: 'urd.codeEditor', params: { path: item.file, line: item.line } });
  }
</script>

<div class="forge-what-if">
  <div class="forge-what-if__header">
    <span class="forge-what-if__title">What If</span>
    {#if selectedSymbol}<span class="forge-what-if__symbol">{selectedSymbol} removed?</span>{/if}
  </div>
  <div class="forge-what-if__results">
    {#if !selectedSymbol}
      <div class="forge-what-if__empty">Select a symbol to analyse removal impact</div>
    {:else if impacts.length === 0}
      <div class="forge-what-if__empty">No downstream impacts detected</div>
    {:else}
      {#each impacts as item}
        <button class="forge-what-if__item" onclick={() => navigateToItem(item)} type="button">
          <span class="forge-what-if__severity forge-what-if__severity--{item.severity}">{item.severity}</span>
          <span class="forge-what-if__desc">{item.description}</span>
          <span class="forge-what-if__location">{item.file}:{item.line}</span>
        </button>
      {/each}
    {/if}
  </div>
</div>

<style>
  .forge-what-if { display: flex; flex-direction: column; width: 100%; height: 100%; overflow: hidden; font-family: var(--forge-font-family-ui); }
  .forge-what-if__header { display: flex; align-items: center; gap: var(--forge-space-sm); padding: var(--forge-space-md); background-color: var(--forge-bg-secondary); border-bottom: 1px solid var(--forge-border-zone); }
  .forge-what-if__title { font-size: var(--forge-font-size-xs); font-weight: 600; color: var(--forge-text-secondary); text-transform: uppercase; letter-spacing: 0.05em; }
  .forge-what-if__symbol { font-family: var(--forge-font-family-mono); font-size: var(--forge-font-size-xs); color: var(--forge-accent-primary); }
  .forge-what-if__results { flex: 1; min-height: 0; overflow-y: auto; }
  .forge-what-if__empty { display: flex; align-items: center; justify-content: center; padding: var(--forge-space-xl); color: var(--forge-text-muted); font-size: var(--forge-font-size-sm); }
  .forge-what-if__item { display: flex; align-items: center; gap: var(--forge-space-sm); width: 100%; padding: var(--forge-space-xs) var(--forge-space-md); background: none; border: none; border-bottom: 1px solid var(--forge-border-subtle, rgba(255,255,255,0.04)); cursor: pointer; font-family: var(--forge-font-family-ui); font-size: var(--forge-font-size-sm); color: var(--forge-text-primary); text-align: left; }
  .forge-what-if__item:hover { background-color: var(--forge-table-row-hover); }
  .forge-what-if__severity { padding: 0 5px; border-radius: var(--forge-radius-sm); font-size: 10px; font-weight: 600; text-transform: uppercase; line-height: 18px; flex-shrink: 0; }
  .forge-what-if__severity--high { background: #e94560; color: #fff; }
  .forge-what-if__severity--medium { background: #e6a817; color: #fff; }
  .forge-what-if__severity--low { background: #888; color: #fff; }
  .forge-what-if__desc { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .forge-what-if__location { flex-shrink: 0; color: var(--forge-text-muted); font-family: var(--forge-font-family-mono); font-size: var(--forge-font-size-xs); }
</style>
