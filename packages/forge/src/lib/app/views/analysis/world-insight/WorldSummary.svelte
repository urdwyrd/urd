<script lang="ts">
  /**
   * WorldSummary — always-visible compact metadata header for the World Insight panel.
   * Two-column grid showing world name, start/entry, counts, diagnostics, compile time.
   */

  import type { UrdWorld, FactSet } from '$lib/app/compiler/types';
  import type { WorldStats } from '$lib/app/projections/world-stats';

  interface Props {
    world: UrdWorld | null;
    factSet: FactSet | null;
    worldStats: WorldStats | null;
    onBadgeClick: (sectionId: string) => void;
    onNavigate: (path: string, line: number) => void;
  }

  let { world, factSet, worldStats, onBadgeClick, onNavigate }: Props = $props();

  let typeCount = $derived(Object.keys(world?.types ?? {}).length);
  let ruleCount = $derived(world?.rules ? Object.keys(world.rules).length : 0);
  let sectionCount = $derived.by(() => {
    if (!factSet) return 0;
    const sections = new Set<string>();
    for (const c of factSet.choices) sections.add(c.section);
    for (const j of factSet.jumps) sections.add(j.from_section);
    return sections.size;
  });
</script>

<div class="forge-world-insight-summary">
  {#if world}
    <div class="forge-world-insight-summary__title">
      {world.world?.name ?? '(unnamed world)'}
    </div>
    <div class="forge-world-insight-summary__grid">
      {#if world.world?.start}
        <span class="forge-world-insight-summary__label">Start</span>
        <button class="forge-world-insight-summary__link" onclick={() => onBadgeClick('locations')}>
          {world.world.start}
        </button>
      {/if}
      {#if world.world?.entry}
        <span class="forge-world-insight-summary__label">Entry</span>
        <button class="forge-world-insight-summary__link" onclick={() => onBadgeClick('dialogue')}>
          {world.world.entry}
        </button>
      {/if}
    </div>
    <div class="forge-world-insight-summary__badges">
      <button class="forge-world-insight-summary__badge" onclick={() => onBadgeClick('entities')}>
        {world.entities.length} entities
      </button>
      <button class="forge-world-insight-summary__badge" onclick={() => onBadgeClick('locations')}>
        {world.locations.length} locations
      </button>
      <button class="forge-world-insight-summary__badge" onclick={() => onBadgeClick('dialogue')}>
        {sectionCount} sections
      </button>
      <button class="forge-world-insight-summary__badge" onclick={() => onBadgeClick('entities')}>
        {typeCount} types
      </button>
      {#if ruleCount > 0}
        <button class="forge-world-insight-summary__badge" onclick={() => onBadgeClick('rules')}>
          {ruleCount} rules
        </button>
      {/if}
      {#if worldStats}
        <span class="forge-world-insight-summary__badge forge-world-insight-summary__badge--static">
          {worldStats.factCount} facts
        </span>
      {/if}
    </div>
    {#if worldStats}
      <div class="forge-world-insight-summary__diagnostics">
        {#if worldStats.errorCount > 0}
          <button
            class="forge-world-insight-summary__diag-badge forge-world-insight-summary__diag-badge--error"
            onclick={() => onBadgeClick('diagnostics')}
          >
            {worldStats.errorCount} errors
          </button>
        {/if}
        {#if worldStats.warningCount > 0}
          <button
            class="forge-world-insight-summary__diag-badge forge-world-insight-summary__diag-badge--warning"
            onclick={() => onBadgeClick('diagnostics')}
          >
            {worldStats.warningCount} warnings
          </button>
        {/if}
        {#if worldStats.infoCount > 0}
          <button
            class="forge-world-insight-summary__diag-badge forge-world-insight-summary__diag-badge--info"
            onclick={() => onBadgeClick('diagnostics')}
          >
            {worldStats.infoCount} info
          </button>
        {/if}
        <span class="forge-world-insight-summary__compile-time">
          {worldStats.compileDurationMs}ms
        </span>
      </div>
    {/if}
  {:else}
    <div class="forge-world-insight-summary__empty">No compilation data</div>
  {/if}
</div>

<style>
  .forge-world-insight-summary {
    padding: var(--forge-space-sm) var(--forge-space-md);
    border-bottom: 1px solid var(--forge-border-zone);
  }

  .forge-world-insight-summary__title {
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-md);
    font-weight: 700;
    color: var(--forge-text-primary);
    margin-bottom: var(--forge-space-xs);
  }

  .forge-world-insight-summary__grid {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: var(--forge-space-xs) var(--forge-space-sm);
    margin-bottom: var(--forge-space-xs);
  }

  .forge-world-insight-summary__label {
    font-family: var(--forge-font-family-mono);
    font-size: 10px;
    color: var(--forge-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }

  .forge-world-insight-summary__link {
    background: none;
    border: none;
    padding: 0;
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-xs);
    color: var(--forge-text-secondary);
    cursor: pointer;
    text-align: left;
  }

  .forge-world-insight-summary__link:hover {
    color: var(--forge-text-primary);
    text-decoration: underline;
  }

  .forge-world-insight-summary__badges {
    display: flex;
    flex-wrap: wrap;
    gap: var(--forge-space-xs);
    margin-bottom: var(--forge-space-xs);
  }

  .forge-world-insight-summary__badge {
    font-family: var(--forge-font-family-mono);
    font-size: 10px;
    padding: 1px var(--forge-space-xs);
    border-radius: var(--forge-radius-sm);
    background-color: color-mix(in srgb, var(--forge-text-muted) 12%, transparent);
    color: var(--forge-text-secondary);
    border: none;
    cursor: pointer;
    line-height: 16px;
  }

  .forge-world-insight-summary__badge:hover {
    background-color: color-mix(in srgb, var(--forge-text-muted) 25%, transparent);
  }

  .forge-world-insight-summary__badge--static {
    cursor: default;
  }

  .forge-world-insight-summary__badge--static:hover {
    background-color: color-mix(in srgb, var(--forge-text-muted) 12%, transparent);
  }

  .forge-world-insight-summary__diagnostics {
    display: flex;
    align-items: center;
    gap: var(--forge-space-xs);
  }

  .forge-world-insight-summary__diag-badge {
    font-family: var(--forge-font-family-mono);
    font-size: 10px;
    padding: 1px var(--forge-space-xs);
    border-radius: var(--forge-radius-sm);
    border: none;
    cursor: pointer;
    line-height: 16px;
  }

  .forge-world-insight-summary__diag-badge--error {
    background-color: color-mix(in srgb, var(--forge-status-error) 15%, transparent);
    color: var(--forge-status-error);
  }

  .forge-world-insight-summary__diag-badge--warning {
    background-color: color-mix(in srgb, var(--forge-status-warning) 15%, transparent);
    color: var(--forge-status-warning);
  }

  .forge-world-insight-summary__diag-badge--info {
    background-color: color-mix(in srgb, var(--forge-status-info, var(--forge-text-muted)) 15%, transparent);
    color: var(--forge-status-info, var(--forge-text-muted));
  }

  .forge-world-insight-summary__compile-time {
    font-family: var(--forge-font-family-mono);
    font-size: 10px;
    color: var(--forge-text-muted);
    margin-left: auto;
  }

  .forge-world-insight-summary__empty {
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-sm);
    color: var(--forge-text-muted);
  }
</style>
