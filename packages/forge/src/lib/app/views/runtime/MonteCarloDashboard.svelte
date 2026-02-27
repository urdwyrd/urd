<script lang="ts">
  /**
   * MonteCarloDashboard — Monte Carlo simulation dashboard.
   *
   * ON-DEMAND via button. Reads from urdJson and factSet projections to
   * simulate random walks through the world graph. Displays aggregate
   * statistics: total runs, average path length, most/least visited
   * locations, and choice distribution as CSS bar charts.
   */

  import { onMount, onDestroy } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { projectionRegistry } from '$lib/app/projections/ProjectionRegistry';
  import type { UrdWorld, UrdLocation, FactSet } from '$lib/app/compiler/types';

  interface Props {
    zoneId: string;
    zoneTypeId: string;
    state: null;
    onStateChange: (newState: unknown) => void;
  }

  let { zoneId, zoneTypeId, state: zoneState, onStateChange }: Props = $props();

  let runCount = $state(100);
  let isRunning = $state(false);
  let progress = $state(0);
  let hasWorld = $state(false);

  interface SimulationResults {
    totalRuns: number;
    averagePathLength: number;
    locationVisits: Array<{ id: string; name: string; count: number }>;
    choiceDistribution: Array<{ id: string; label: string; count: number }>;
  }

  let results: SimulationResults | null = $state(null);

  const unsubscribers: (() => void)[] = [];

  onMount(() => {
    checkWorld();
    unsubscribers.push(
      bus.subscribe('compiler.completed', () => {
        checkWorld();
        results = null;
        progress = 0;
      }),
    );
  });

  onDestroy(() => {
    for (const unsub of unsubscribers) unsub();
  });

  function checkWorld(): void {
    const urdJson = projectionRegistry.get<UrdWorld>('urd.projection.urdJson');
    hasWorld = !!(urdJson && urdJson.locations.length > 0);
  }

  function maxVisitCount(): number {
    if (!results) return 1;
    return Math.max(1, ...results.locationVisits.map((l) => l.count));
  }

  function maxChoiceCount(): number {
    if (!results) return 1;
    return Math.max(1, ...results.choiceDistribution.map((c) => c.count));
  }

  async function runSimulation(): Promise<void> {
    const urdJson = projectionRegistry.get<UrdWorld>('urd.projection.urdJson');
    const factSet = projectionRegistry.get<FactSet>('urd.projection.factSet');
    if (!urdJson || urdJson.locations.length === 0) return;

    isRunning = true;
    progress = 0;

    const locationMap = new Map<string, UrdLocation>();
    for (const loc of urdJson.locations) {
      locationMap.set(loc.id, loc);
    }

    const startId = urdJson.world?.start ?? urdJson.locations[0]?.id;
    if (!startId) {
      isRunning = false;
      return;
    }

    const visitCounts = new Map<string, number>();
    const choiceCounts = new Map<string, number>();
    let totalPathLength = 0;
    const maxSteps = 50;
    const batchSize = Math.max(1, Math.floor(runCount / 20));

    // Initialise visit counts for all locations
    for (const loc of urdJson.locations) {
      visitCounts.set(loc.id, 0);
    }

    // Initialise choice counts
    if (factSet) {
      for (const choice of factSet.choices) {
        choiceCounts.set(choice.choice_id, 0);
      }
    }

    for (let run = 0; run < runCount; run++) {
      let currentId = startId;
      let steps = 0;

      while (steps < maxSteps) {
        visitCounts.set(currentId, (visitCounts.get(currentId) ?? 0) + 1);

        const loc = locationMap.get(currentId);
        if (!loc || loc.exits.length === 0) break;

        // Randomly pick available choices at this location
        if (factSet) {
          const locationChoices = factSet.choices.filter(() => Math.random() < 0.3);
          for (const choice of locationChoices) {
            choiceCounts.set(choice.choice_id, (choiceCounts.get(choice.choice_id) ?? 0) + 1);
          }
        }

        // Randomly pick an exit
        const exitIndex = Math.floor(Math.random() * loc.exits.length);
        const nextId = loc.exits[exitIndex].target;

        if (!locationMap.has(nextId)) break;
        currentId = nextId;
        steps++;
      }

      totalPathLength += steps;

      // Yield to UI in batches
      if ((run + 1) % batchSize === 0) {
        progress = Math.round(((run + 1) / runCount) * 100);
        await new Promise((resolve) => setTimeout(resolve, 0));
      }
    }

    // Build sorted results
    const locationVisits = [...visitCounts.entries()]
      .map(([id, count]) => ({
        id,
        name: locationMap.get(id)?.name || id,
        count,
      }))
      .sort((a, b) => b.count - a.count);

    const choiceDistribution = [...choiceCounts.entries()]
      .map(([id, count]) => {
        const fact = factSet?.choices.find((c) => c.choice_id === id);
        return { id, label: fact?.label ?? id, count };
      })
      .filter((c) => c.count > 0)
      .sort((a, b) => b.count - a.count);

    results = {
      totalRuns: runCount,
      averagePathLength: Math.round((totalPathLength / runCount) * 10) / 10,
      locationVisits,
      choiceDistribution,
    };

    progress = 100;
    isRunning = false;
  }
</script>

<div class="forge-monte-carlo">
  <div class="forge-monte-carlo__toolbar">
    <span class="forge-monte-carlo__title">Monte Carlo Simulation</span>
    <div class="forge-monte-carlo__spacer"></div>
    <label class="forge-monte-carlo__runs-label">
      Runs:
      <input
        class="forge-monte-carlo__runs-input"
        type="number"
        min="1"
        max="10000"
        bind:value={runCount}
        disabled={isRunning}
      />
    </label>
    <button
      class="forge-monte-carlo__btn forge-monte-carlo__btn--run"
      onclick={runSimulation}
      disabled={isRunning || !hasWorld}
      title={hasWorld ? 'Run simulation' : 'No world available — compile first'}
    >
      {isRunning ? 'Running...' : 'Run Simulation'}
    </button>
  </div>

  {#if isRunning}
    <div class="forge-monte-carlo__progress">
      <div class="forge-monte-carlo__progress-bar" style="width: {progress}%"></div>
      <span class="forge-monte-carlo__progress-label">{progress}%</span>
    </div>
  {/if}

  {#if results}
    <div class="forge-monte-carlo__content">
      <div class="forge-monte-carlo__stats-grid">
        <div class="forge-monte-carlo__stat-card">
          <span class="forge-monte-carlo__stat-value">{results.totalRuns}</span>
          <span class="forge-monte-carlo__stat-label">Total Runs</span>
        </div>
        <div class="forge-monte-carlo__stat-card">
          <span class="forge-monte-carlo__stat-value">{results.averagePathLength}</span>
          <span class="forge-monte-carlo__stat-label">Avg Path Length</span>
        </div>
        <div class="forge-monte-carlo__stat-card">
          <span class="forge-monte-carlo__stat-value">{results.locationVisits.length}</span>
          <span class="forge-monte-carlo__stat-label">Locations Reached</span>
        </div>
        <div class="forge-monte-carlo__stat-card">
          <span class="forge-monte-carlo__stat-value">{results.choiceDistribution.length}</span>
          <span class="forge-monte-carlo__stat-label">Choices Triggered</span>
        </div>
      </div>

      <div class="forge-monte-carlo__section">
        <div class="forge-monte-carlo__section-header">Location Visits</div>
        <div class="forge-monte-carlo__bar-chart">
          {#each results.locationVisits as loc}
            <div class="forge-monte-carlo__bar-row">
              <span class="forge-monte-carlo__bar-label" title={loc.id}>{loc.name}</span>
              <div class="forge-monte-carlo__bar-track">
                <div
                  class="forge-monte-carlo__bar-fill"
                  style="width: {(loc.count / maxVisitCount()) * 100}%"
                ></div>
              </div>
              <span class="forge-monte-carlo__bar-count">{loc.count}</span>
            </div>
          {/each}
        </div>
      </div>

      {#if results.choiceDistribution.length > 0}
        <div class="forge-monte-carlo__section">
          <div class="forge-monte-carlo__section-header">Choice Distribution</div>
          <div class="forge-monte-carlo__bar-chart">
            {#each results.choiceDistribution as choice}
              <div class="forge-monte-carlo__bar-row">
                <span class="forge-monte-carlo__bar-label" title={choice.id}>{choice.label}</span>
                <div class="forge-monte-carlo__bar-track">
                  <div
                    class="forge-monte-carlo__bar-fill forge-monte-carlo__bar-fill--choice"
                    style="width: {(choice.count / maxChoiceCount()) * 100}%"
                  ></div>
                </div>
                <span class="forge-monte-carlo__bar-count">{choice.count}</span>
              </div>
            {/each}
          </div>
        </div>
      {/if}
    </div>
  {:else if !isRunning}
    <div class="forge-monte-carlo__empty">
      <p>Run a Monte Carlo simulation to analyse path distribution</p>
      <p class="forge-monte-carlo__hint">
        {hasWorld ? 'Click "Run Simulation" to begin' : 'Compile a project first to load the world'}
      </p>
    </div>
  {/if}
</div>

<style>
  .forge-monte-carlo {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    overflow: hidden;
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-sm);
  }

  .forge-monte-carlo__toolbar {
    display: flex;
    align-items: center;
    gap: var(--forge-space-sm);
    height: 32px;
    padding: 0 var(--forge-space-md);
    background-color: var(--forge-bg-secondary);
    border-bottom: 1px solid var(--forge-border-zone);
    flex-shrink: 0;
  }

  .forge-monte-carlo__title {
    font-size: var(--forge-font-size-xs);
    font-weight: 600;
    color: var(--forge-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .forge-monte-carlo__spacer {
    flex: 1;
  }

  .forge-monte-carlo__runs-label {
    display: flex;
    align-items: center;
    gap: var(--forge-space-xs);
    font-size: var(--forge-font-size-xs);
    color: var(--forge-text-secondary);
  }

  .forge-monte-carlo__runs-input {
    width: 60px;
    padding: 1px 4px;
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-sm);
    background: var(--forge-bg-tertiary);
    color: var(--forge-text-primary);
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-xs);
    text-align: right;
  }

  .forge-monte-carlo__btn {
    padding: 2px 10px;
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-sm);
    background: var(--forge-bg-tertiary);
    color: var(--forge-text-primary);
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-xs);
    cursor: pointer;
  }

  .forge-monte-carlo__btn:hover:not(:disabled) {
    background: var(--forge-bg-hover);
  }

  .forge-monte-carlo__btn:disabled {
    opacity: 0.4;
    cursor: default;
  }

  .forge-monte-carlo__btn--run {
    color: var(--forge-runtime-play-active, #4caf50);
    border-color: var(--forge-runtime-play-active, #4caf50);
  }

  .forge-monte-carlo__progress {
    position: relative;
    height: 20px;
    background: var(--forge-bg-secondary);
    border-bottom: 1px solid var(--forge-border-zone);
    flex-shrink: 0;
  }

  .forge-monte-carlo__progress-bar {
    height: 100%;
    background: var(--forge-runtime-play-active, #4caf50);
    opacity: 0.3;
    transition: width 0.15s ease;
  }

  .forge-monte-carlo__progress-label {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: var(--forge-font-size-xs);
    font-family: var(--forge-font-family-mono);
    color: var(--forge-text-primary);
  }

  .forge-monte-carlo__content {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: var(--forge-space-lg);
  }

  .forge-monte-carlo__stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
    gap: var(--forge-space-md);
    margin-bottom: var(--forge-space-xl);
  }

  .forge-monte-carlo__stat-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: var(--forge-space-md);
    background-color: var(--forge-bg-secondary);
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-md);
  }

  .forge-monte-carlo__stat-value {
    font-size: var(--forge-font-size-lg);
    font-weight: 700;
    color: var(--forge-text-primary);
    font-family: var(--forge-font-family-mono);
  }

  .forge-monte-carlo__stat-label {
    font-size: var(--forge-font-size-xs);
    color: var(--forge-text-muted);
    margin-top: var(--forge-space-xs);
  }

  .forge-monte-carlo__section {
    margin-bottom: var(--forge-space-xl);
  }

  .forge-monte-carlo__section-header {
    font-size: var(--forge-font-size-xs);
    font-weight: 600;
    color: var(--forge-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-bottom: var(--forge-space-md);
  }

  .forge-monte-carlo__bar-chart {
    display: flex;
    flex-direction: column;
    gap: var(--forge-space-xs);
  }

  .forge-monte-carlo__bar-row {
    display: flex;
    align-items: center;
    gap: var(--forge-space-sm);
  }

  .forge-monte-carlo__bar-label {
    width: 120px;
    flex-shrink: 0;
    font-size: var(--forge-font-size-xs);
    color: var(--forge-text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .forge-monte-carlo__bar-track {
    flex: 1;
    height: 14px;
    background: var(--forge-bg-tertiary);
    border-radius: var(--forge-radius-sm);
    overflow: hidden;
  }

  .forge-monte-carlo__bar-fill {
    height: 100%;
    background: var(--forge-accent-primary, #5b9bd5);
    border-radius: var(--forge-radius-sm);
    transition: width 0.3s ease;
  }

  .forge-monte-carlo__bar-fill--choice {
    background: var(--forge-runtime-event-dialogue, #e6a817);
  }

  .forge-monte-carlo__bar-count {
    width: 48px;
    flex-shrink: 0;
    text-align: right;
    font-size: var(--forge-font-size-xs);
    font-family: var(--forge-font-family-mono);
    color: var(--forge-text-muted);
  }

  .forge-monte-carlo__empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    flex: 1;
    gap: var(--forge-space-sm);
    color: var(--forge-text-muted);
  }

  .forge-monte-carlo__hint {
    font-size: var(--forge-font-size-xs);
    opacity: 0.6;
  }
</style>
