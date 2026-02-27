<script lang="ts">
  /**
   * DiffView — compares current compilation against a saved baseline.
   *
   * Has a "Save Baseline" button to capture the current compiler output stats.
   * Compares current vs baseline showing added, removed, and changed items.
   * Stores the baseline in local $state (not a projection).
   */

  import { onMount, onDestroy } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { projectionRegistry } from '$lib/app/projections/ProjectionRegistry';
  import type { WorldStats } from '$lib/app/projections/world-stats';
  import type { SymbolTable, SymbolTableEntry } from '$lib/app/compiler/types';

  interface Props {
    zoneId: string;
    zoneTypeId: string;
    state: null;
    onStateChange: (newState: unknown) => void;
  }

  let { zoneId, zoneTypeId, state: zoneState, onStateChange }: Props = $props();

  interface BaselineSnapshot {
    timestamp: number;
    stats: WorldStats;
    symbolNames: Set<string>;
    symbolList: SymbolTableEntry[];
  }

  interface DiffItem {
    name: string;
    kind: string;
    change: 'added' | 'removed' | 'unchanged';
  }

  let baseline = $state<BaselineSnapshot | null>(null);
  let currentStats = $state<WorldStats | null>(null);
  let currentSymbols = $state<SymbolTable | null>(null);

  const unsubscribers: (() => void)[] = [];

  onMount(() => {
    refreshData();
    unsubscribers.push(bus.subscribe('compiler.completed', refreshData));
  });

  onDestroy(() => {
    for (const unsub of unsubscribers) unsub();
  });

  function refreshData(): void {
    currentStats = projectionRegistry.get<WorldStats>('urd.projection.worldStats');
    currentSymbols = projectionRegistry.get<SymbolTable>('urd.projection.symbolTable');
  }

  function saveBaseline(): void {
    if (!currentStats || !currentSymbols) return;
    const names = new Set(currentSymbols.entries.map((e) => `${e.kind}:${e.name}`));
    baseline = {
      timestamp: Date.now(),
      stats: { ...currentStats },
      symbolNames: names,
      symbolList: [...currentSymbols.entries],
    };
  }

  function clearBaseline(): void {
    baseline = null;
  }

  let baselineDate = $derived(
    baseline ? new Date(baseline.timestamp).toLocaleTimeString() : '',
  );

  interface StatDiff {
    label: string;
    baselineVal: number | string;
    currentVal: number | string;
    changed: boolean;
  }

  let statDiffs = $derived.by((): StatDiff[] => {
    if (!baseline || !currentStats) return [];
    const b = baseline.stats;
    const c = currentStats;
    const pairs: Array<[string, number | string, number | string]> = [
      ['Entities', b.entityCount, c.entityCount],
      ['Locations', b.locationCount, c.locationCount],
      ['Properties', b.propertyCount, c.propertyCount],
      ['Rules', b.ruleCount, c.ruleCount],
      ['Facts', b.factCount, c.factCount],
      ['Symbols', b.symbolCount, c.symbolCount],
      ['Files', b.fileCount, c.fileCount],
      ['Errors', b.errorCount, c.errorCount],
      ['Warnings', b.warningCount, c.warningCount],
    ];
    return pairs.map(([label, baselineVal, currentVal]) => ({
      label,
      baselineVal,
      currentVal,
      changed: baselineVal !== currentVal,
    }));
  });

  let symbolDiffs = $derived.by((): DiffItem[] => {
    if (!baseline || !currentSymbols) return [];
    const currentNames = new Set(currentSymbols.entries.map((e) => `${e.kind}:${e.name}`));
    const items: DiffItem[] = [];

    // Added symbols
    for (const entry of currentSymbols.entries) {
      const key = `${entry.kind}:${entry.name}`;
      if (!baseline.symbolNames.has(key)) {
        items.push({ name: entry.name, kind: entry.kind, change: 'added' });
      }
    }

    // Removed symbols
    for (const entry of baseline.symbolList) {
      const key = `${entry.kind}:${entry.name}`;
      if (!currentNames.has(key)) {
        items.push({ name: entry.name, kind: entry.kind, change: 'removed' });
      }
    }

    return items.sort((a, b) => {
      if (a.change !== b.change) return a.change === 'added' ? -1 : 1;
      return a.name.localeCompare(b.name);
    });
  });

  function diffColour(change: string): string {
    if (change === 'added') return 'var(--forge-status-success)';
    if (change === 'removed') return 'var(--forge-status-error)';
    return 'var(--forge-text-muted)';
  }

  function diffPrefix(change: string): string {
    if (change === 'added') return '+';
    if (change === 'removed') return '-';
    return ' ';
  }

  function statChangeColour(diff: StatDiff): string {
    if (!diff.changed) return 'var(--forge-text-muted)';
    const bv = typeof diff.baselineVal === 'number' ? diff.baselineVal : 0;
    const cv = typeof diff.currentVal === 'number' ? diff.currentVal : 0;
    return cv > bv ? 'var(--forge-status-success)' : 'var(--forge-status-error)';
  }
</script>

<div class="forge-analysis-diff">
  <div class="forge-analysis-diff__toolbar">
    <button
      class="forge-analysis-diff__btn"
      onclick={saveBaseline}
      disabled={!currentStats}
    >
      Save Baseline
    </button>
    {#if baseline}
      <button class="forge-analysis-diff__btn" onclick={clearBaseline}>
        Clear Baseline
      </button>
      <span class="forge-analysis-diff__timestamp">
        Baseline: {baselineDate}
      </span>
    {/if}
  </div>

  {#if !baseline}
    <div class="forge-analysis-diff__empty">
      Save a baseline snapshot to compare against future compilations
    </div>
  {:else if !currentStats}
    <div class="forge-analysis-diff__empty">
      No current compilation data available
    </div>
  {:else}
    <div class="forge-analysis-diff__content">
      <div class="forge-analysis-diff__section-header">
        Stat Changes
      </div>
      <div class="forge-analysis-diff__stats">
        {#each statDiffs as diff}
          <div
            class="forge-analysis-diff__stat-row"
            class:forge-analysis-diff__stat-row--changed={diff.changed}
          >
            <span class="forge-analysis-diff__stat-label">{diff.label}</span>
            <span class="forge-analysis-diff__stat-values" style:color={statChangeColour(diff)}>
              {diff.baselineVal} -> {diff.currentVal}
            </span>
          </div>
        {/each}
      </div>

      {#if symbolDiffs.length > 0}
        <div class="forge-analysis-diff__section-header">
          Symbol Changes ({symbolDiffs.length})
        </div>
        <div class="forge-analysis-diff__symbols">
          {#each symbolDiffs as item}
            <div class="forge-analysis-diff__symbol-row" style:color={diffColour(item.change)}>
              <span class="forge-analysis-diff__symbol-prefix">{diffPrefix(item.change)}</span>
              <span class="forge-analysis-diff__symbol-kind">{item.kind}</span>
              <span class="forge-analysis-diff__symbol-name">{item.name}</span>
            </div>
          {/each}
        </div>
      {:else}
        <div class="forge-analysis-diff__no-symbol-changes">
          No symbol additions or removals
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .forge-analysis-diff {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    overflow-y: auto;
    font-family: var(--forge-font-family-ui);
  }

  .forge-analysis-diff__toolbar {
    display: flex;
    align-items: center;
    gap: var(--forge-space-sm);
    padding: var(--forge-space-sm) var(--forge-space-md);
    border-bottom: 1px solid var(--forge-border-zone);
    flex-shrink: 0;
  }

  .forge-analysis-diff__btn {
    padding: var(--forge-space-xs) var(--forge-space-md);
    background-color: var(--forge-bg-secondary);
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-sm);
    color: var(--forge-text-primary);
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-sm);
    cursor: pointer;
  }

  .forge-analysis-diff__btn:hover:not(:disabled) {
    background-color: var(--forge-table-row-hover);
  }

  .forge-analysis-diff__btn:disabled {
    opacity: 0.5;
    cursor: default;
  }

  .forge-analysis-diff__timestamp {
    font-size: var(--forge-font-size-xs);
    color: var(--forge-text-muted);
    margin-left: auto;
  }

  .forge-analysis-diff__empty {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    flex: 1;
    color: var(--forge-text-muted);
    font-size: var(--forge-font-size-md);
    padding: var(--forge-space-xl);
    text-align: center;
  }

  .forge-analysis-diff__content {
    padding: var(--forge-space-md);
  }

  .forge-analysis-diff__section-header {
    font-size: var(--forge-font-size-xs);
    font-weight: 600;
    color: var(--forge-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    padding: var(--forge-space-xs) 0;
    border-bottom: 1px solid var(--forge-border-zone);
    margin-bottom: var(--forge-space-sm);
    margin-top: var(--forge-space-md);
  }

  .forge-analysis-diff__section-header:first-child {
    margin-top: 0;
  }

  .forge-analysis-diff__stats {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .forge-analysis-diff__stat-row {
    display: flex;
    justify-content: space-between;
    padding: var(--forge-space-xs) var(--forge-space-sm);
    border-radius: var(--forge-radius-sm);
    font-size: var(--forge-font-size-sm);
  }

  .forge-analysis-diff__stat-row--changed {
    background-color: var(--forge-bg-secondary);
  }

  .forge-analysis-diff__stat-label {
    color: var(--forge-text-secondary);
  }

  .forge-analysis-diff__stat-values {
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-xs);
  }

  .forge-analysis-diff__symbols {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .forge-analysis-diff__symbol-row {
    display: flex;
    align-items: center;
    gap: var(--forge-space-sm);
    padding: var(--forge-space-xs) var(--forge-space-sm);
    font-size: var(--forge-font-size-sm);
    font-family: var(--forge-font-family-mono);
  }

  .forge-analysis-diff__symbol-prefix {
    font-weight: 700;
    width: 1em;
    text-align: center;
  }

  .forge-analysis-diff__symbol-kind {
    font-size: var(--forge-font-size-xs);
    opacity: 0.7;
    min-width: 60px;
  }

  .forge-analysis-diff__symbol-name {
    font-weight: 500;
  }

  .forge-analysis-diff__no-symbol-changes {
    font-size: var(--forge-font-size-sm);
    color: var(--forge-text-muted);
    padding: var(--forge-space-sm);
  }
</style>
