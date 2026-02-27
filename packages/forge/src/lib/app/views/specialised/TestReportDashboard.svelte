<script lang="ts">
  /**
   * TestReportDashboard — test report viewer.
   *
   * Displays a placeholder with compiler test status. Could read from
   * a test report projection in the future. Shows test report header
   * and placeholder message.
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

  let hasCompiled = $state(false);
  let errorCount = $state(0);
  let warningCount = $state(0);

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
    if (stats) {
      hasCompiled = true;
      errorCount = stats.errorCount;
      warningCount = stats.warningCount;
    }
  }

  function statusColour(): string {
    if (!hasCompiled) return 'var(--forge-text-muted)';
    if (errorCount > 0) return 'var(--forge-status-error, #e94560)';
    if (warningCount > 0) return 'var(--forge-status-warning, #e6a817)';
    return 'var(--forge-runtime-play-active, #4caf50)';
  }

  function statusText(): string {
    if (!hasCompiled) return 'No compilation data';
    if (errorCount > 0) return `${errorCount} error${errorCount !== 1 ? 's' : ''}`;
    if (warningCount > 0) return `${warningCount} warning${warningCount !== 1 ? 's' : ''}`;
    return 'All checks passed';
  }
</script>

<div class="forge-test-report">
  <div class="forge-test-report__toolbar">
    <span class="forge-test-report__title">Test Report Dashboard</span>
  </div>

  <div class="forge-test-report__content">
    <div class="forge-test-report__status-card">
      <div class="forge-test-report__status-indicator" style="background-color: {statusColour()}"></div>
      <div class="forge-test-report__status-info">
        <span class="forge-test-report__status-text" style="color: {statusColour()}">
          {statusText()}
        </span>
        <span class="forge-test-report__status-hint">
          {hasCompiled ? 'Based on latest compilation diagnostics' : 'Compile a project to see test status'}
        </span>
      </div>
    </div>

    <div class="forge-test-report__placeholder">
      <p class="forge-test-report__placeholder-text">
        Full test report integration is planned for a future release.
        This dashboard will display comprehensive test results, coverage
        metrics, and validation status from the Urd compiler test suite.
      </p>
    </div>
  </div>
</div>

<style>
  .forge-test-report {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    overflow: hidden;
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-sm);
  }

  .forge-test-report__toolbar {
    display: flex;
    align-items: center;
    gap: var(--forge-space-sm);
    height: 28px;
    padding: 0 var(--forge-space-md);
    background-color: var(--forge-bg-secondary);
    border-bottom: 1px solid var(--forge-border-zone);
    flex-shrink: 0;
  }

  .forge-test-report__title {
    font-size: var(--forge-font-size-xs);
    font-weight: 600;
    color: var(--forge-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .forge-test-report__content {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: var(--forge-space-lg);
    display: flex;
    flex-direction: column;
    gap: var(--forge-space-xl);
  }

  .forge-test-report__status-card {
    display: flex;
    align-items: center;
    gap: var(--forge-space-md);
    padding: var(--forge-space-lg);
    background: var(--forge-bg-secondary);
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-md);
  }

  .forge-test-report__status-indicator {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .forge-test-report__status-info {
    display: flex;
    flex-direction: column;
    gap: var(--forge-space-xs);
  }

  .forge-test-report__status-text {
    font-weight: 600;
    font-size: var(--forge-font-size-md);
  }

  .forge-test-report__status-hint {
    font-size: var(--forge-font-size-xs);
    color: var(--forge-text-muted);
  }

  .forge-test-report__placeholder {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--forge-space-xl);
    border: 1px dashed var(--forge-border-zone);
    border-radius: var(--forge-radius-md);
  }

  .forge-test-report__placeholder-text {
    color: var(--forge-text-muted);
    font-size: var(--forge-font-size-sm);
    line-height: 1.6;
    max-width: 400px;
    text-align: center;
  }
</style>
