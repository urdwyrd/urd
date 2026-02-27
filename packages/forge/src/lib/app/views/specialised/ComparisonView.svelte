<script lang="ts">
  /**
   * ComparisonView — side-by-side comparison of world stats.
   *
   * Has a "Pin Current" button to save a baseline snapshot. Then shows
   * current vs baseline comparison. Reads worldStats projection.
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

  interface StatRow {
    label: string;
    current: number | string;
    baseline: number | string;
    delta: string;
    deltaClass: string;
  }

  let currentStats: WorldStats | null = $state(null);
  let baselineStats: WorldStats | null = $state(null);
  let baselinePinnedAt: string | null = $state(null);

  const unsubscribers: (() => void)[] = [];

  onMount(() => {
    refreshCurrent();
    unsubscribers.push(bus.subscribe('compiler.completed', refreshCurrent));
  });

  onDestroy(() => {
    for (const unsub of unsubscribers) unsub();
  });

  function refreshCurrent(): void {
    currentStats = projectionRegistry.get<WorldStats>('urd.projection.worldStats');
  }

  function pinBaseline(): void {
    if (!currentStats) return;
    // Deep clone the stats
    baselineStats = JSON.parse(JSON.stringify(currentStats));
    baselinePinnedAt = new Date().toLocaleTimeString();
  }

  function clearBaseline(): void {
    baselineStats = null;
    baselinePinnedAt = null;
  }

  function computeDelta(current: number, baseline: number): { text: string; cls: string } {
    const diff = current - baseline;
    if (diff === 0) return { text: '—', cls: '' };
    if (diff > 0) return { text: `+${diff}`, cls: 'forge-comparison__delta--up' };
    return { text: String(diff), cls: 'forge-comparison__delta--down' };
  }

  let rows = $derived.by((): StatRow[] => {
    if (!currentStats) return [];

    const fields: Array<{ label: string; key: keyof WorldStats }> = [
      { label: 'Entities', key: 'entityCount' },
      { label: 'Locations', key: 'locationCount' },
      { label: 'Exits', key: 'exitCount' },
      { label: 'Properties', key: 'propertyCount' },
      { label: 'Rules', key: 'ruleCount' },
      { label: 'Facts', key: 'factCount' },
      { label: 'Symbols', key: 'symbolCount' },
      { label: 'Files', key: 'fileCount' },
      { label: 'Errors', key: 'errorCount' },
      { label: 'Warnings', key: 'warningCount' },
      { label: 'Compile (ms)', key: 'compileDurationMs' },
    ];

    return fields.map((f) => {
      const currentVal = currentStats![f.key] as number;
      const baselineVal = baselineStats ? (baselineStats[f.key] as number) : null;
      const delta = baselineVal !== null ? computeDelta(currentVal, baselineVal) : { text: '—', cls: '' };

      return {
        label: f.label,
        current: currentVal,
        baseline: baselineVal !== null ? baselineVal : '—',
        delta: delta.text,
        deltaClass: delta.cls,
      };
    });
  });
</script>

<div class="forge-comparison">
  <div class="forge-comparison__toolbar">
    <span class="forge-comparison__title">Comparison View</span>
    <div class="forge-comparison__spacer"></div>
    {#if currentStats}
      <button
        class="forge-comparison__btn forge-comparison__btn--pin"
        onclick={pinBaseline}
        title="Pin current stats as baseline"
      >
        Pin Current
      </button>
    {/if}
    {#if baselineStats}
      <button
        class="forge-comparison__btn"
        onclick={clearBaseline}
        title="Clear baseline"
      >
        Clear Baseline
      </button>
    {/if}
  </div>

  {#if !currentStats}
    <div class="forge-comparison__empty">
      <p>No compilation data available</p>
      <p class="forge-comparison__hint">Compile a project to see world stats and pin a baseline for comparison</p>
    </div>
  {:else}
    <div class="forge-comparison__content">
      {#if baselinePinnedAt}
        <div class="forge-comparison__baseline-info">
          Baseline pinned at {baselinePinnedAt}
        </div>
      {/if}

      <table class="forge-comparison__table">
        <thead>
          <tr>
            <th class="forge-comparison__th">Metric</th>
            <th class="forge-comparison__th forge-comparison__th--right">Current</th>
            {#if baselineStats}
              <th class="forge-comparison__th forge-comparison__th--right">Baseline</th>
              <th class="forge-comparison__th forge-comparison__th--right">Delta</th>
            {/if}
          </tr>
        </thead>
        <tbody>
          {#each rows as row}
            <tr class="forge-comparison__row">
              <td class="forge-comparison__td forge-comparison__td--label">{row.label}</td>
              <td class="forge-comparison__td forge-comparison__td--value">{row.current}</td>
              {#if baselineStats}
                <td class="forge-comparison__td forge-comparison__td--value">{row.baseline}</td>
                <td class="forge-comparison__td forge-comparison__td--delta {row.deltaClass}">
                  {row.delta}
                </td>
              {/if}
            </tr>
          {/each}
        </tbody>
      </table>

      {#if !baselineStats}
        <div class="forge-comparison__no-baseline">
          Click "Pin Current" to save a baseline for comparison
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .forge-comparison {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    overflow: hidden;
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-sm);
  }

  .forge-comparison__toolbar {
    display: flex;
    align-items: center;
    gap: var(--forge-space-sm);
    height: 32px;
    padding: 0 var(--forge-space-md);
    background-color: var(--forge-bg-secondary);
    border-bottom: 1px solid var(--forge-border-zone);
    flex-shrink: 0;
  }

  .forge-comparison__title {
    font-size: var(--forge-font-size-xs);
    font-weight: 600;
    color: var(--forge-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .forge-comparison__spacer {
    flex: 1;
  }

  .forge-comparison__btn {
    padding: 2px 8px;
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-sm);
    background: var(--forge-bg-tertiary);
    color: var(--forge-text-primary);
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-xs);
    cursor: pointer;
  }

  .forge-comparison__btn:hover {
    background: var(--forge-bg-hover);
  }

  .forge-comparison__btn--pin {
    color: var(--forge-accent-primary, #5b9bd5);
    border-color: var(--forge-accent-primary, #5b9bd5);
  }

  .forge-comparison__empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    flex: 1;
    gap: var(--forge-space-sm);
    color: var(--forge-text-muted);
  }

  .forge-comparison__hint {
    font-size: var(--forge-font-size-xs);
    opacity: 0.6;
  }

  .forge-comparison__content {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: var(--forge-space-lg);
  }

  .forge-comparison__baseline-info {
    font-size: var(--forge-font-size-xs);
    color: var(--forge-text-muted);
    margin-bottom: var(--forge-space-md);
    font-style: italic;
  }

  .forge-comparison__table {
    width: 100%;
    border-collapse: collapse;
    margin-bottom: var(--forge-space-lg);
  }

  .forge-comparison__th {
    padding: var(--forge-space-sm) var(--forge-space-md);
    font-size: var(--forge-font-size-xs);
    font-weight: 600;
    color: var(--forge-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    text-align: left;
    border-bottom: 1px solid var(--forge-border-zone);
  }

  .forge-comparison__th--right {
    text-align: right;
  }

  .forge-comparison__row:hover {
    background: var(--forge-bg-hover);
  }

  .forge-comparison__td {
    padding: var(--forge-space-sm) var(--forge-space-md);
    border-bottom: 1px solid var(--forge-border-subtle, rgba(255, 255, 255, 0.04));
  }

  .forge-comparison__td--label {
    color: var(--forge-text-secondary);
  }

  .forge-comparison__td--value {
    text-align: right;
    font-family: var(--forge-font-family-mono);
    color: var(--forge-text-primary);
  }

  .forge-comparison__td--delta {
    text-align: right;
    font-family: var(--forge-font-family-mono);
    font-weight: 600;
  }

  :global(.forge-comparison__delta--up) {
    color: var(--forge-runtime-play-active, #4caf50);
  }

  :global(.forge-comparison__delta--down) {
    color: var(--forge-graph-node-unreachable, #e94560);
  }

  .forge-comparison__no-baseline {
    text-align: center;
    color: var(--forge-text-muted);
    font-size: var(--forge-font-size-xs);
    padding: var(--forge-space-lg);
    border: 1px dashed var(--forge-border-zone);
    border-radius: var(--forge-radius-md);
  }
</style>
