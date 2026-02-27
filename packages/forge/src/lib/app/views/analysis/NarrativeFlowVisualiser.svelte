<script lang="ts">
  /**
   * NarrativeFlowVisualiser — displays a vertical list of narrative steps
   * with branching indicators.
   *
   * Reads from the urd.projection.narrativeFlow projection, auto-refreshed
   * on compiler.completed. Shows sections as steps in a vertical flow
   * with choice/jump counts and branch connections.
   */

  import { onMount, onDestroy } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { projectionRegistry } from '$lib/app/projections/ProjectionRegistry';
  import type { NarrativeFlowData, NarrativeStep } from '$lib/app/projections/narrative-flow';

  interface Props {
    zoneId: string;
    zoneTypeId: string;
    state: null;
    onStateChange: (newState: unknown) => void;
  }

  let { zoneId, zoneTypeId, state: zoneState, onStateChange }: Props = $props();

  let data = $state<NarrativeFlowData | null>(null);

  const unsubscribers: (() => void)[] = [];

  onMount(() => {
    refreshData();
    unsubscribers.push(bus.subscribe('compiler.completed', refreshData));
  });

  onDestroy(() => {
    for (const unsub of unsubscribers) unsub();
  });

  function refreshData(): void {
    data = projectionRegistry.get<NarrativeFlowData>('urd.projection.narrativeFlow');
  }

  let terminalCount = $derived(
    (data?.steps ?? []).filter((s) => s.isTerminal).length,
  );

  let branchingCount = $derived(
    (data?.steps ?? []).filter((s) => s.choiceCount > 1 || s.jumpCount > 1).length,
  );

  // Build outgoing branches per section for display
  let outgoingBranches = $derived.by(() => {
    const map = new Map<string, Array<{ to: string; label: string }>>();
    for (const branch of (data?.branches ?? [])) {
      const list = map.get(branch.from) ?? [];
      list.push({ to: branch.to, label: branch.label });
      map.set(branch.from, list);
    }
    return map;
  });

  function stepIndicator(step: NarrativeStep): string {
    if (step.isTerminal) return '  ';
    if (step.choiceCount > 1 || step.jumpCount > 1) return '>';
    return '|';
  }

  function stepColour(step: NarrativeStep): string {
    if (step.isTerminal) return 'var(--forge-status-error)';
    if (step.choiceCount > 1 || step.jumpCount > 1) return 'var(--forge-status-warning)';
    return 'var(--forge-text-secondary)';
  }
</script>

<div class="forge-analysis-narrative">
  {#if !data || data.steps.length === 0}
    <div class="forge-analysis-narrative__empty">
      {#if !data}
        No compilation data available
      {:else}
        No narrative sections found
      {/if}
    </div>
  {:else}
    <div class="forge-analysis-narrative__content">
      <div class="forge-analysis-narrative__summary">
        {data.steps.length} sections, {data.branches.length} branches,
        {branchingCount} branching point{branchingCount !== 1 ? 's' : ''},
        {terminalCount} terminal{terminalCount !== 1 ? 's' : ''}
      </div>

      <div class="forge-analysis-narrative__flow">
        {#each data.steps as step, i}
          <div class="forge-analysis-narrative__step">
            <div class="forge-analysis-narrative__step-line">
              <span
                class="forge-analysis-narrative__step-indicator"
                style:color={stepColour(step)}
              >
                {stepIndicator(step)}
              </span>
              <span class="forge-analysis-narrative__step-name">
                {step.sectionName}
              </span>
              <div class="forge-analysis-narrative__step-badges">
                {#if step.choiceCount > 0}
                  <span class="forge-analysis-narrative__badge forge-analysis-narrative__badge--choice">
                    {step.choiceCount} choice{step.choiceCount !== 1 ? 's' : ''}
                  </span>
                {/if}
                {#if step.jumpCount > 0}
                  <span class="forge-analysis-narrative__badge forge-analysis-narrative__badge--jump">
                    {step.jumpCount} jump{step.jumpCount !== 1 ? 's' : ''}
                  </span>
                {/if}
                {#if step.isTerminal}
                  <span class="forge-analysis-narrative__badge forge-analysis-narrative__badge--terminal">
                    terminal
                  </span>
                {/if}
              </div>
            </div>

            {#if outgoingBranches.has(step.sectionId)}
              <div class="forge-analysis-narrative__branches">
                {#each outgoingBranches.get(step.sectionId) ?? [] as branch}
                  <div class="forge-analysis-narrative__branch">
                    <span class="forge-analysis-narrative__branch-arrow">-></span>
                    <span class="forge-analysis-narrative__branch-target">{branch.to}</span>
                    <span class="forge-analysis-narrative__branch-label">{branch.label}</span>
                  </div>
                {/each}
              </div>
            {/if}

            {#if i < data.steps.length - 1}
              <div class="forge-analysis-narrative__connector"></div>
            {/if}
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>

<style>
  .forge-analysis-narrative {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    overflow-y: auto;
    font-family: var(--forge-font-family-ui);
  }

  .forge-analysis-narrative__empty {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    color: var(--forge-text-muted);
    font-size: var(--forge-font-size-md);
  }

  .forge-analysis-narrative__content {
    padding: var(--forge-space-md);
  }

  .forge-analysis-narrative__summary {
    font-size: var(--forge-font-size-sm);
    color: var(--forge-text-secondary);
    margin-bottom: var(--forge-space-lg);
  }

  .forge-analysis-narrative__flow {
    display: flex;
    flex-direction: column;
    padding-left: var(--forge-space-md);
  }

  .forge-analysis-narrative__step {
    display: flex;
    flex-direction: column;
  }

  .forge-analysis-narrative__step-line {
    display: flex;
    align-items: center;
    gap: var(--forge-space-sm);
    padding: var(--forge-space-xs) 0;
  }

  .forge-analysis-narrative__step-indicator {
    font-family: var(--forge-font-family-mono);
    font-weight: 700;
    font-size: var(--forge-font-size-md);
    width: 1.5em;
    text-align: center;
    flex-shrink: 0;
  }

  .forge-analysis-narrative__step-name {
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-sm);
    font-weight: 500;
    color: var(--forge-text-primary);
  }

  .forge-analysis-narrative__step-badges {
    display: flex;
    gap: var(--forge-space-xs);
    margin-left: auto;
  }

  .forge-analysis-narrative__badge {
    font-size: 9px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    padding: 1px var(--forge-space-xs);
    border-radius: var(--forge-radius-sm);
  }

  .forge-analysis-narrative__badge--choice {
    color: rgba(100, 180, 255, 0.9);
    background-color: rgba(100, 180, 255, 0.15);
  }

  .forge-analysis-narrative__badge--jump {
    color: rgba(180, 120, 255, 0.9);
    background-color: rgba(180, 120, 255, 0.15);
  }

  .forge-analysis-narrative__badge--terminal {
    color: var(--forge-status-error);
    background-color: rgba(255, 100, 100, 0.15);
  }

  .forge-analysis-narrative__branches {
    padding-left: calc(1.5em + var(--forge-space-sm));
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .forge-analysis-narrative__branch {
    display: flex;
    align-items: center;
    gap: var(--forge-space-xs);
    font-size: var(--forge-font-size-xs);
  }

  .forge-analysis-narrative__branch-arrow {
    color: var(--forge-text-muted);
    font-family: var(--forge-font-family-mono);
    font-weight: 700;
  }

  .forge-analysis-narrative__branch-target {
    color: var(--forge-text-secondary);
    font-family: var(--forge-font-family-mono);
  }

  .forge-analysis-narrative__branch-label {
    color: var(--forge-text-muted);
    font-style: italic;
    font-size: var(--forge-font-size-xs);
  }

  .forge-analysis-narrative__connector {
    width: 1px;
    height: var(--forge-space-sm);
    background-color: var(--forge-border-zone);
    margin-left: calc(0.75em);
  }
</style>
