<script lang="ts">
  /**
   * EnumCoverage — displays coverage of enum property values.
   *
   * Reads from the urd.projection.enumCoverage projection, auto-refreshed
   * on compiler.completed. Lists enum properties with coverage bars
   * showing visited vs total values.
   */

  import { onMount, onDestroy } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { projectionRegistry } from '$lib/app/projections/ProjectionRegistry';
  import type { EnumCoverageEntry } from '$lib/app/projections/enum-coverage';

  interface Props {
    zoneId: string;
    zoneTypeId: string;
    state: null;
    onStateChange: (newState: unknown) => void;
  }

  let { zoneId, zoneTypeId, state: zoneState, onStateChange }: Props = $props();

  let entries: EnumCoverageEntry[] = $state([]);

  const unsubscribers: (() => void)[] = [];

  onMount(() => {
    refreshData();
    unsubscribers.push(bus.subscribe('compiler.completed', refreshData));
  });

  onDestroy(() => {
    for (const unsub of unsubscribers) unsub();
  });

  function refreshData(): void {
    entries = projectionRegistry.get<EnumCoverageEntry[]>('urd.projection.enumCoverage') ?? [];
  }

  let sorted = $derived.by(() => {
    return [...entries].sort((a, b) => a.coveragePct - b.coveragePct);
  });

  function coverageColour(pct: number): string {
    if (pct >= 80) return 'var(--forge-status-success)';
    if (pct >= 50) return 'var(--forge-status-warning)';
    return 'var(--forge-status-error)';
  }
</script>

<div class="forge-analysis-enum-coverage">
  {#if entries.length === 0}
    <div class="forge-analysis-enum-coverage__empty">
      No enum properties found
    </div>
  {:else}
    <div class="forge-analysis-enum-coverage__list">
      {#each sorted as entry}
        <div class="forge-analysis-enum-coverage__item">
          <div class="forge-analysis-enum-coverage__header">
            <span class="forge-analysis-enum-coverage__name">
              {entry.entityType}.{entry.property}
            </span>
            <span class="forge-analysis-enum-coverage__pct" style:color={coverageColour(entry.coveragePct)}>
              {entry.coveragePct}%
            </span>
          </div>

          <div class="forge-analysis-enum-coverage__bar-track">
            <div
              class="forge-analysis-enum-coverage__bar-fill"
              style:width="{entry.coveragePct}%"
              style:background-color={coverageColour(entry.coveragePct)}
            ></div>
          </div>

          <div class="forge-analysis-enum-coverage__detail">
            <span class="forge-analysis-enum-coverage__count">
              {entry.usedValues.length} / {entry.totalValues} values used
            </span>
            {#if entry.unusedValues.length > 0}
              <span class="forge-analysis-enum-coverage__unused">
                Unused: {entry.unusedValues.join(', ')}
              </span>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .forge-analysis-enum-coverage {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    overflow-y: auto;
    font-family: var(--forge-font-family-ui);
  }

  .forge-analysis-enum-coverage__empty {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    color: var(--forge-text-muted);
    font-size: var(--forge-font-size-md);
  }

  .forge-analysis-enum-coverage__list {
    padding: var(--forge-space-md);
    display: flex;
    flex-direction: column;
    gap: var(--forge-space-md);
  }

  .forge-analysis-enum-coverage__item {
    padding: var(--forge-space-sm) var(--forge-space-md);
    background-color: var(--forge-bg-secondary);
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-md);
  }

  .forge-analysis-enum-coverage__header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--forge-space-xs);
  }

  .forge-analysis-enum-coverage__name {
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-sm);
    font-weight: 500;
    color: var(--forge-text-primary);
  }

  .forge-analysis-enum-coverage__pct {
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-sm);
    font-weight: 700;
  }

  .forge-analysis-enum-coverage__bar-track {
    width: 100%;
    height: 6px;
    background-color: var(--forge-border-zone);
    border-radius: 3px;
    overflow: hidden;
    margin-bottom: var(--forge-space-xs);
  }

  .forge-analysis-enum-coverage__bar-fill {
    height: 100%;
    border-radius: 3px;
    transition: width 0.2s ease;
  }

  .forge-analysis-enum-coverage__detail {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .forge-analysis-enum-coverage__count {
    font-size: var(--forge-font-size-xs);
    color: var(--forge-text-secondary);
  }

  .forge-analysis-enum-coverage__unused {
    font-size: var(--forge-font-size-xs);
    color: var(--forge-text-muted);
    font-family: var(--forge-font-family-mono);
  }
</style>
