<script lang="ts">
  /**
   * CircularDependency — lists detected cycles in location exits and section jumps.
   *
   * Reads from the urd.projection.circularDependency projection, auto-refreshed
   * on compiler.completed. Displays each cycle with its path and kind indicator.
   */

  import { onMount, onDestroy } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { projectionRegistry } from '$lib/app/projections/ProjectionRegistry';
  import type { CircularDependencyResult, CycleEntry } from '$lib/app/projections/circular-dependency';

  interface Props {
    zoneId: string;
    zoneTypeId: string;
    state: null;
    onStateChange: (newState: unknown) => void;
  }

  let { zoneId, zoneTypeId, state: zoneState, onStateChange }: Props = $props();

  let result = $state<CircularDependencyResult | null>(null);

  const unsubscribers: (() => void)[] = [];

  onMount(() => {
    refreshData();
    unsubscribers.push(bus.subscribe('compiler.completed', refreshData));
  });

  onDestroy(() => {
    for (const unsub of unsubscribers) unsub();
  });

  function refreshData(): void {
    result = projectionRegistry.get<CircularDependencyResult>('urd.projection.circularDependency');
  }

  let locationCycles = $derived(
    (result?.cycles ?? []).filter((c) => c.kind === 'location'),
  );

  let sectionCycles = $derived(
    (result?.cycles ?? []).filter((c) => c.kind === 'section'),
  );

  function formatPath(cycle: CycleEntry): string {
    return cycle.path.join(' -> ');
  }

  function kindLabel(kind: string): string {
    return kind === 'location' ? 'Location' : 'Section';
  }
</script>

<div class="forge-analysis-circular">
  {#if !result || result.totalCycles === 0}
    <div class="forge-analysis-circular__empty">
      {#if !result}
        No compilation data available
      {:else}
        No circular dependencies detected
      {/if}
    </div>
  {:else}
    <div class="forge-analysis-circular__list">
      <div class="forge-analysis-circular__summary">
        {result.totalCycles} cycle{result.totalCycles !== 1 ? 's' : ''} detected
      </div>

      {#if locationCycles.length > 0}
        <div class="forge-analysis-circular__section-header">
          Location Cycles ({locationCycles.length})
        </div>
        {#each locationCycles as cycle, i}
          <div class="forge-analysis-circular__cycle">
            <span class="forge-analysis-circular__cycle-badge forge-analysis-circular__cycle-badge--location">
              {kindLabel(cycle.kind)}
            </span>
            <span class="forge-analysis-circular__cycle-path">
              {formatPath(cycle)}
            </span>
          </div>
        {/each}
      {/if}

      {#if sectionCycles.length > 0}
        <div class="forge-analysis-circular__section-header">
          Section Cycles ({sectionCycles.length})
        </div>
        {#each sectionCycles as cycle, i}
          <div class="forge-analysis-circular__cycle">
            <span class="forge-analysis-circular__cycle-badge forge-analysis-circular__cycle-badge--section">
              {kindLabel(cycle.kind)}
            </span>
            <span class="forge-analysis-circular__cycle-path">
              {formatPath(cycle)}
            </span>
          </div>
        {/each}
      {/if}
    </div>
  {/if}
</div>

<style>
  .forge-analysis-circular {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    overflow-y: auto;
    font-family: var(--forge-font-family-ui);
  }

  .forge-analysis-circular__empty {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    color: var(--forge-text-muted);
    font-size: var(--forge-font-size-md);
  }

  .forge-analysis-circular__list {
    padding: var(--forge-space-md);
  }

  .forge-analysis-circular__summary {
    font-size: var(--forge-font-size-sm);
    color: var(--forge-text-secondary);
    margin-bottom: var(--forge-space-md);
  }

  .forge-analysis-circular__section-header {
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

  .forge-analysis-circular__section-header:first-child {
    margin-top: 0;
  }

  .forge-analysis-circular__cycle {
    display: flex;
    align-items: flex-start;
    gap: var(--forge-space-sm);
    padding: var(--forge-space-sm) var(--forge-space-md);
    background-color: var(--forge-bg-secondary);
    border: 1px solid var(--forge-status-warning);
    border-radius: var(--forge-radius-md);
    margin-bottom: var(--forge-space-sm);
  }

  .forge-analysis-circular__cycle-badge {
    flex-shrink: 0;
    font-size: var(--forge-font-size-xs);
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    padding: 2px var(--forge-space-xs);
    border-radius: var(--forge-radius-sm);
  }

  .forge-analysis-circular__cycle-badge--location {
    color: var(--forge-status-warning);
    background-color: rgba(255, 160, 100, 0.15);
  }

  .forge-analysis-circular__cycle-badge--section {
    color: var(--forge-status-error);
    background-color: rgba(255, 100, 100, 0.15);
  }

  .forge-analysis-circular__cycle-path {
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-sm);
    color: var(--forge-text-primary);
    word-break: break-all;
    line-height: 1.5;
  }
</style>
