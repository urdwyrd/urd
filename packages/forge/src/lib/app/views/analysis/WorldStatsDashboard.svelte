<script lang="ts">
  /**
   * WorldStatsDashboard — CSS grid of stat cards from the worldStats projection.
   */

  import { onMount, onDestroy } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { projectionRegistry } from '$lib/app/projections/ProjectionRegistry';
  import type { WorldStats } from '$lib/app/projections/world-stats';

  interface Props {
    zoneId: string;
    zoneTypeId: string;
    state: null;
    onStateChange: (newState: unknown) => void;
  }

  let { zoneId, zoneTypeId, state: zoneState, onStateChange }: Props = $props();

  let stats: WorldStats | null = $state(null);

  const unsubscribers: (() => void)[] = [];

  onMount(() => {
    refreshData();
    unsubscribers.push(bus.subscribe('compiler.completed', refreshData));
  });

  onDestroy(() => {
    for (const unsub of unsubscribers) unsub();
  });

  function refreshData(): void {
    stats = projectionRegistry.get<WorldStats>('urd.projection.worldStats');
  }

  interface StatCard {
    label: string;
    value: number | string;
    colour?: string;
  }

  let cards = $derived.by((): StatCard[] => {
    if (!stats) return [];
    return [
      { label: 'Entities', value: stats.entityCount },
      { label: 'Locations', value: stats.locationCount },
      { label: 'Exits', value: stats.exitCount },
      { label: 'Properties', value: stats.propertyCount },
      { label: 'Rules', value: stats.ruleCount },
      { label: 'Facts', value: stats.factCount },
      { label: 'Symbols', value: stats.symbolCount },
      { label: 'Files', value: stats.fileCount },
      { label: 'Errors', value: stats.errorCount, colour: stats.errorCount > 0 ? 'var(--forge-status-error)' : undefined },
      { label: 'Warnings', value: stats.warningCount, colour: stats.warningCount > 0 ? 'var(--forge-status-warning)' : undefined },
      { label: 'Compile', value: `${stats.compileDurationMs}ms` },
    ];
  });
</script>

<div class="forge-world-stats">
  {#if stats}
    <div class="forge-world-stats__grid">
      {#each cards as card}
        <div class="forge-world-stats__card">
          <span class="forge-world-stats__value" style:color={card.colour}>{card.value}</span>
          <span class="forge-world-stats__label">{card.label}</span>
        </div>
      {/each}
    </div>

    {#if stats.phaseTimings.length > 0}
      <div class="forge-world-stats__timings">
        <span class="forge-world-stats__timings-title">Phase Timings</span>
        <div class="forge-world-stats__timings-list">
          {#each stats.phaseTimings as timing}
            <div class="forge-world-stats__timing-row">
              <span class="forge-world-stats__timing-phase">{timing.phase}</span>
              <span class="forge-world-stats__timing-value">{timing.durationMs}ms</span>
            </div>
          {/each}
        </div>
      </div>
    {/if}
  {:else}
    <div class="forge-world-stats__empty">
      No compilation data available
    </div>
  {/if}
</div>

<style>
  .forge-world-stats {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    overflow-y: auto;
    padding: var(--forge-space-lg);
    font-family: var(--forge-font-family-ui);
  }

  .forge-world-stats__grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(100px, 1fr));
    gap: var(--forge-space-md);
    margin-bottom: var(--forge-space-xl);
  }

  .forge-world-stats__card {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: var(--forge-space-md);
    background-color: var(--forge-bg-secondary);
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-md);
  }

  .forge-world-stats__value {
    font-size: var(--forge-font-size-lg);
    font-weight: 700;
    color: var(--forge-text-primary);
    font-family: var(--forge-font-family-mono);
  }

  .forge-world-stats__label {
    font-size: var(--forge-font-size-xs);
    color: var(--forge-text-muted);
    margin-top: var(--forge-space-xs);
  }

  .forge-world-stats__timings {
    padding: var(--forge-space-md);
    background-color: var(--forge-bg-secondary);
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-md);
  }

  .forge-world-stats__timings-title {
    font-size: var(--forge-font-size-xs);
    font-weight: 600;
    color: var(--forge-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    display: block;
    margin-bottom: var(--forge-space-sm);
  }

  .forge-world-stats__timings-list {
    display: flex;
    flex-direction: column;
    gap: var(--forge-space-xs);
  }

  .forge-world-stats__timing-row {
    display: flex;
    justify-content: space-between;
    font-size: var(--forge-font-size-sm);
  }

  .forge-world-stats__timing-phase {
    color: var(--forge-text-secondary);
  }

  .forge-world-stats__timing-value {
    color: var(--forge-text-primary);
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-xs);
  }

  .forge-world-stats__empty {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    color: var(--forge-text-muted);
    font-size: var(--forge-font-size-md);
  }
</style>
