<script lang="ts">
  /**
   * CompilationPipeline — compiler phase visualisation.
   *
   * Subscribes to compiler.completed bus events to capture timing data.
   * Shows a horizontal bar chart of phase durations. Displays a placeholder
   * until compilation data is available.
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

  interface PhaseBar {
    name: string;
    durationMs: number;
    percentage: number;
  }

  let phases: PhaseBar[] = $state([]);
  let totalMs = $state(0);
  let compileTimestamp: string | null = $state(null);

  const unsubscribers: (() => void)[] = [];

  onMount(() => {
    refreshData();
    unsubscribers.push(bus.subscribe('compiler.completed', refreshData));
  });

  onDestroy(() => {
    for (const unsub of unsubscribers) unsub();
  });

  function refreshData(): void {
    const stats = projectionRegistry.get<WorldStats>('urd.projection.worldStats');
    if (!stats || stats.phaseTimings.length === 0) {
      phases = [];
      totalMs = 0;
      compileTimestamp = null;
      return;
    }

    totalMs = stats.compileDurationMs;
    compileTimestamp = new Date().toLocaleTimeString();

    const maxDuration = Math.max(1, ...stats.phaseTimings.map((p) => p.durationMs));

    phases = stats.phaseTimings.map((p) => ({
      name: p.phase,
      durationMs: p.durationMs,
      percentage: (p.durationMs / maxDuration) * 100,
    }));
  }

  function phaseColour(index: number): string {
    const colours = [
      'var(--forge-runtime-event-move, #5b9bd5)',
      'var(--forge-runtime-event-set, #4caf50)',
      'var(--forge-runtime-event-dialogue, #e6a817)',
      'var(--forge-runtime-event-section, #9b59b6)',
      'var(--forge-accent-primary, #5b9bd5)',
    ];
    return colours[index % colours.length];
  }
</script>

<div class="forge-compilation-pipeline">
  <div class="forge-compilation-pipeline__toolbar">
    <span class="forge-compilation-pipeline__title">Compilation Pipeline</span>
    <div class="forge-compilation-pipeline__spacer"></div>
    {#if totalMs > 0}
      <span class="forge-compilation-pipeline__total">{totalMs}ms total</span>
    {/if}
  </div>

  {#if phases.length === 0}
    <div class="forge-compilation-pipeline__empty">
      <p>Compile a project to see pipeline timings</p>
      <p class="forge-compilation-pipeline__hint">
        Phase durations will appear here after compilation
      </p>
    </div>
  {:else}
    <div class="forge-compilation-pipeline__content">
      {#if compileTimestamp}
        <div class="forge-compilation-pipeline__timestamp">
          Last compiled at {compileTimestamp}
        </div>
      {/if}

      <div class="forge-compilation-pipeline__chart">
        {#each phases as phase, i}
          <div class="forge-compilation-pipeline__phase">
            <span class="forge-compilation-pipeline__phase-name">{phase.name}</span>
            <div class="forge-compilation-pipeline__phase-bar-track">
              <div
                class="forge-compilation-pipeline__phase-bar-fill"
                style="width: {phase.percentage}%; background-color: {phaseColour(i)}"
              ></div>
            </div>
            <span class="forge-compilation-pipeline__phase-duration">{phase.durationMs}ms</span>
          </div>
        {/each}
      </div>

      <div class="forge-compilation-pipeline__summary">
        <div class="forge-compilation-pipeline__summary-row">
          <span class="forge-compilation-pipeline__summary-label">Phases</span>
          <span class="forge-compilation-pipeline__summary-value">{phases.length}</span>
        </div>
        <div class="forge-compilation-pipeline__summary-row">
          <span class="forge-compilation-pipeline__summary-label">Total duration</span>
          <span class="forge-compilation-pipeline__summary-value">{totalMs}ms</span>
        </div>
        <div class="forge-compilation-pipeline__summary-row">
          <span class="forge-compilation-pipeline__summary-label">Average per phase</span>
          <span class="forge-compilation-pipeline__summary-value">
            {Math.round(totalMs / phases.length)}ms
          </span>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .forge-compilation-pipeline {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    overflow: hidden;
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-sm);
  }

  .forge-compilation-pipeline__toolbar {
    display: flex;
    align-items: center;
    gap: var(--forge-space-sm);
    height: 28px;
    padding: 0 var(--forge-space-md);
    background-color: var(--forge-bg-secondary);
    border-bottom: 1px solid var(--forge-border-zone);
    flex-shrink: 0;
  }

  .forge-compilation-pipeline__title {
    font-size: var(--forge-font-size-xs);
    font-weight: 600;
    color: var(--forge-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .forge-compilation-pipeline__spacer {
    flex: 1;
  }

  .forge-compilation-pipeline__total {
    font-size: var(--forge-font-size-xs);
    font-family: var(--forge-font-family-mono);
    color: var(--forge-text-muted);
  }

  .forge-compilation-pipeline__empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    flex: 1;
    gap: var(--forge-space-sm);
    color: var(--forge-text-muted);
  }

  .forge-compilation-pipeline__hint {
    font-size: var(--forge-font-size-xs);
    opacity: 0.6;
  }

  .forge-compilation-pipeline__content {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: var(--forge-space-lg);
  }

  .forge-compilation-pipeline__timestamp {
    font-size: var(--forge-font-size-xs);
    color: var(--forge-text-muted);
    margin-bottom: var(--forge-space-lg);
  }

  .forge-compilation-pipeline__chart {
    display: flex;
    flex-direction: column;
    gap: var(--forge-space-sm);
    margin-bottom: var(--forge-space-xl);
  }

  .forge-compilation-pipeline__phase {
    display: flex;
    align-items: center;
    gap: var(--forge-space-sm);
  }

  .forge-compilation-pipeline__phase-name {
    width: 80px;
    flex-shrink: 0;
    font-size: var(--forge-font-size-xs);
    color: var(--forge-text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .forge-compilation-pipeline__phase-bar-track {
    flex: 1;
    height: 16px;
    background: var(--forge-bg-tertiary);
    border-radius: var(--forge-radius-sm);
    overflow: hidden;
  }

  .forge-compilation-pipeline__phase-bar-fill {
    height: 100%;
    border-radius: var(--forge-radius-sm);
    transition: width 0.3s ease;
  }

  .forge-compilation-pipeline__phase-duration {
    width: 48px;
    flex-shrink: 0;
    text-align: right;
    font-size: var(--forge-font-size-xs);
    font-family: var(--forge-font-family-mono);
    color: var(--forge-text-muted);
  }

  .forge-compilation-pipeline__summary {
    padding: var(--forge-space-md);
    background: var(--forge-bg-secondary);
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-md);
  }

  .forge-compilation-pipeline__summary-row {
    display: flex;
    justify-content: space-between;
    padding: var(--forge-space-xs) 0;
  }

  .forge-compilation-pipeline__summary-label {
    font-size: var(--forge-font-size-xs);
    color: var(--forge-text-secondary);
  }

  .forge-compilation-pipeline__summary-value {
    font-size: var(--forge-font-size-xs);
    font-family: var(--forge-font-family-mono);
    color: var(--forge-text-primary);
    font-weight: 600;
  }
</style>
