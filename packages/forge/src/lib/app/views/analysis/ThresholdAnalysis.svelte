<script lang="ts">
  /**
   * ThresholdAnalysis — lists numeric properties with bounds and suspicious writes.
   *
   * Reads from the urd.projection.thresholdAnalysis projection, auto-refreshed
   * on compiler.completed. Shows each bounded property with its range and any
   * writes that fall outside the declared min/max.
   */

  import { onMount, onDestroy } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { projectionRegistry } from '$lib/app/projections/ProjectionRegistry';
  import { navigationBroker } from '$lib/framework/navigation/NavigationBroker';
  import type { ThresholdEntry } from '$lib/app/projections/threshold-analysis';

  interface Props {
    zoneId: string;
    zoneTypeId: string;
    state: null;
    onStateChange: (newState: unknown) => void;
  }

  let { zoneId, zoneTypeId, state: zoneState, onStateChange }: Props = $props();

  let entries: ThresholdEntry[] = $state([]);

  const unsubscribers: (() => void)[] = [];

  onMount(() => {
    refreshData();
    unsubscribers.push(bus.subscribe('compiler.completed', refreshData));
  });

  onDestroy(() => {
    for (const unsub of unsubscribers) unsub();
  });

  function refreshData(): void {
    entries = projectionRegistry.get<ThresholdEntry[]>('urd.projection.thresholdAnalysis') ?? [];
  }

  let withIssues = $derived(entries.filter((e) => e.suspiciousWrites.length > 0));
  let withoutIssues = $derived(entries.filter((e) => e.suspiciousWrites.length === 0));

  function boundsLabel(entry: ThresholdEntry): string {
    const parts: string[] = [];
    if (entry.min !== null) parts.push(`min: ${entry.min}`);
    if (entry.max !== null) parts.push(`max: ${entry.max}`);
    return parts.join(', ');
  }

  function goToSource(file: string, line: number): void {
    if (file) {
      navigationBroker.navigate({
        targetViewId: 'urd.codeEditor',
        params: { path: file, line },
      });
    }
  }
</script>

<div class="forge-analysis-threshold">
  {#if entries.length === 0}
    <div class="forge-analysis-threshold__empty">
      No numeric properties with bounds found
    </div>
  {:else}
    <div class="forge-analysis-threshold__list">
      {#if withIssues.length > 0}
        <div class="forge-analysis-threshold__section-header">
          Suspicious Writes ({withIssues.length})
        </div>
        {#each withIssues as entry}
          <div class="forge-analysis-threshold__item forge-analysis-threshold__item--warn">
            <div class="forge-analysis-threshold__item-header">
              <span class="forge-analysis-threshold__name">
                {entry.entityType}.{entry.property}
              </span>
              <span class="forge-analysis-threshold__bounds">
                [{boundsLabel(entry)}]
              </span>
            </div>
            <div class="forge-analysis-threshold__writes">
              {#each entry.suspiciousWrites as write}
                <button
                  class="forge-analysis-threshold__write-entry"
                  onclick={() => goToSource(write.file, write.line)}
                >
                  <span class="forge-analysis-threshold__write-value">{write.value}</span>
                  <span class="forge-analysis-threshold__write-loc">{write.file}:{write.line}</span>
                </button>
              {/each}
            </div>
          </div>
        {/each}
      {/if}

      {#if withoutIssues.length > 0}
        <div class="forge-analysis-threshold__section-header">
          Within Bounds ({withoutIssues.length})
        </div>
        {#each withoutIssues as entry}
          <div class="forge-analysis-threshold__item">
            <div class="forge-analysis-threshold__item-header">
              <span class="forge-analysis-threshold__name">
                {entry.entityType}.{entry.property}
              </span>
              <span class="forge-analysis-threshold__bounds">
                [{boundsLabel(entry)}]
              </span>
            </div>
          </div>
        {/each}
      {/if}
    </div>
  {/if}
</div>

<style>
  .forge-analysis-threshold {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    overflow-y: auto;
    font-family: var(--forge-font-family-ui);
  }

  .forge-analysis-threshold__empty {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    color: var(--forge-text-muted);
    font-size: var(--forge-font-size-md);
  }

  .forge-analysis-threshold__list {
    padding: var(--forge-space-md);
  }

  .forge-analysis-threshold__section-header {
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

  .forge-analysis-threshold__section-header:first-child {
    margin-top: 0;
  }

  .forge-analysis-threshold__item {
    padding: var(--forge-space-sm) var(--forge-space-md);
    background-color: var(--forge-bg-secondary);
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-md);
    margin-bottom: var(--forge-space-sm);
  }

  .forge-analysis-threshold__item--warn {
    border-color: var(--forge-status-warning);
  }

  .forge-analysis-threshold__item-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .forge-analysis-threshold__name {
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-sm);
    font-weight: 500;
    color: var(--forge-text-primary);
  }

  .forge-analysis-threshold__bounds {
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-xs);
    color: var(--forge-text-muted);
  }

  .forge-analysis-threshold__writes {
    margin-top: var(--forge-space-xs);
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .forge-analysis-threshold__write-entry {
    display: flex;
    justify-content: space-between;
    align-items: center;
    width: 100%;
    padding: var(--forge-space-xs) var(--forge-space-sm);
    background: none;
    border: none;
    border-radius: var(--forge-radius-sm);
    cursor: pointer;
    text-align: left;
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-xs);
    color: var(--forge-text-primary);
  }

  .forge-analysis-threshold__write-entry:hover {
    background-color: var(--forge-table-row-hover);
  }

  .forge-analysis-threshold__write-value {
    font-weight: 600;
    color: var(--forge-status-warning);
    font-family: var(--forge-font-family-mono);
  }

  .forge-analysis-threshold__write-loc {
    color: var(--forge-text-muted);
    font-family: var(--forge-font-family-mono);
  }
</style>
