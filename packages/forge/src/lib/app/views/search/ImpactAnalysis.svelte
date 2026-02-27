<script lang="ts">
  /**
   * ImpactAnalysis — traces downstream effects of changing a property.
   */

  import { onMount, onDestroy } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { projectionRegistry } from '$lib/app/projections/ProjectionRegistry';
  import { selectionContext, type SelectionState } from '$lib/framework/selection/SelectionContext';
  import { navigationBroker } from '$lib/framework/navigation/NavigationBroker';
  import type { FactSet, PropertyDependencyIndex } from '$lib/app/compiler/types';

  interface Props { zoneId: string; zoneTypeId: string; state: null; onStateChange: (newState: unknown) => void; }
  let { zoneId, zoneTypeId, state: zoneState, onStateChange }: Props = $props();

  interface ImpactEntry { description: string; kind: string; file: string; line: number; }

  let selectedSymbol: string | null = $state(null);
  let impacts: ImpactEntry[] = $state([]);
  const unsubscribers: (() => void)[] = [];

  onMount(() => {
    updateFromSelection(selectionContext.state);
    unsubscribers.push(selectionContext.subscribe(updateFromSelection));
    unsubscribers.push(bus.subscribe('compiler.completed', () => { if (selectedSymbol) analyse(selectedSymbol); }));
  });
  onDestroy(() => { for (const unsub of unsubscribers) unsub(); });

  function updateFromSelection(state: SelectionState): void {
    if (state.items.length > 0) { selectedSymbol = state.items[0].id; analyse(selectedSymbol); }
  }

  function analyse(symbolId: string): void {
    const factSet = projectionRegistry.get<FactSet>('urd.projection.factSet');
    const depIndex = projectionRegistry.get<PropertyDependencyIndex>('urd.projection.propertyDependencyIndex');
    if (!factSet || !depIndex) { impacts = []; return; }
    const items: ImpactEntry[] = [];

    // Find reads of this property
    for (const read of factSet.reads) {
      if (read.property === symbolId || `${read.entity_type}.${read.property}` === symbolId) {
        items.push({ description: `Read by ${read.site.kind} "${read.site.id}"`, kind: 'read', file: read.span.file, line: read.span.start_line });
      }
    }
    // Find writes
    for (const write of factSet.writes) {
      if (write.property === symbolId || `${write.entity_type}.${write.property}` === symbolId) {
        items.push({ description: `Written by ${write.site.kind} "${write.site.id}"`, kind: 'write', file: write.span.file, line: write.span.start_line });
      }
    }
    impacts = items;
  }

  function navigateToItem(item: ImpactEntry): void {
    navigationBroker.navigate({ targetViewId: 'urd.codeEditor', params: { path: item.file, line: item.line } });
  }
</script>

<div class="forge-impact">
  <div class="forge-impact__header">
    <span class="forge-impact__title">Impact Analysis</span>
    {#if selectedSymbol}<span class="forge-impact__symbol">{selectedSymbol}</span>{/if}
  </div>
  <div class="forge-impact__results">
    {#if !selectedSymbol}
      <div class="forge-impact__empty">Select a property to trace its impact</div>
    {:else if impacts.length === 0}
      <div class="forge-impact__empty">No downstream effects detected</div>
    {:else}
      {#each impacts as item}
        <button class="forge-impact__item" onclick={() => navigateToItem(item)} type="button">
          <span class="forge-impact__kind">{item.kind}</span>
          <span class="forge-impact__desc">{item.description}</span>
          <span class="forge-impact__location">{item.file}:{item.line}</span>
        </button>
      {/each}
    {/if}
  </div>
</div>

<style>
  .forge-impact { display: flex; flex-direction: column; width: 100%; height: 100%; overflow: hidden; font-family: var(--forge-font-family-ui); }
  .forge-impact__header { display: flex; align-items: center; gap: var(--forge-space-sm); padding: var(--forge-space-md); background-color: var(--forge-bg-secondary); border-bottom: 1px solid var(--forge-border-zone); }
  .forge-impact__title { font-size: var(--forge-font-size-xs); font-weight: 600; color: var(--forge-text-secondary); text-transform: uppercase; letter-spacing: 0.05em; }
  .forge-impact__symbol { font-family: var(--forge-font-family-mono); font-size: var(--forge-font-size-xs); color: var(--forge-accent-primary); }
  .forge-impact__results { flex: 1; min-height: 0; overflow-y: auto; }
  .forge-impact__empty { display: flex; align-items: center; justify-content: center; padding: var(--forge-space-xl); color: var(--forge-text-muted); font-size: var(--forge-font-size-sm); }
  .forge-impact__item { display: flex; align-items: center; gap: var(--forge-space-sm); width: 100%; height: 28px; padding: 0 var(--forge-space-md); background: none; border: none; cursor: pointer; font-family: var(--forge-font-family-ui); font-size: var(--forge-font-size-sm); color: var(--forge-text-primary); text-align: left; }
  .forge-impact__item:hover { background-color: var(--forge-table-row-hover); }
  .forge-impact__kind { padding: 0 5px; border-radius: var(--forge-radius-sm); font-size: 10px; font-weight: 600; text-transform: uppercase; line-height: 18px; flex-shrink: 0; background: var(--forge-bg-tertiary); color: var(--forge-text-muted); }
  .forge-impact__desc { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .forge-impact__location { flex-shrink: 0; color: var(--forge-text-muted); font-family: var(--forge-font-family-mono); font-size: var(--forge-font-size-xs); }
</style>
