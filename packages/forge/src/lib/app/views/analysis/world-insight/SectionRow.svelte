<script lang="ts">
  /**
   * SectionRow — expandable dialogue section row within DialogueOutline.
   * L0: section heading, choice count, jump count, source ref.
   * L1: individual choices with markers, conditions, effects, jumps.
   */

  import type { ChoiceFact, JumpEdge, FactSet } from '$lib/app/compiler/types';

  interface Props {
    sectionId: string;
    choices: ChoiceFact[];
    jumps: JumpEdge[];
    factSet: FactSet;
    expanded: boolean;
    onToggle: (key: string) => void;
    onNavigate: (path: string, line: number) => void;
  }

  let { sectionId, choices, jumps, factSet, expanded, onToggle, onNavigate }: Props = $props();

  // Find the earliest span for source reference
  let sourceRef = $derived.by(() => {
    let earliest: { file: string; line: number } | null = null;
    for (const c of choices) {
      if (!earliest || c.span.start_line < earliest.line) {
        earliest = { file: c.span.file, line: c.span.start_line };
      }
    }
    for (const j of jumps) {
      if (!earliest || j.span.start_line < earliest.line) {
        earliest = { file: j.span.file, line: j.span.start_line };
      }
    }
    return earliest;
  });

  function handleToggle(): void {
    onToggle(`section:${sectionId}`);
  }
</script>

<div class="forge-section-row">
  <button class="forge-section-row__header" onclick={handleToggle}>
    <span class="forge-section-row__toggle">{expanded ? '\u25BE' : '\u25B8'}</span>
    <span class="forge-section-row__id">## {sectionId}</span>
    <span class="forge-section-row__badge">{choices.length} choices</span>
    {#if jumps.length > 0}
      <span class="forge-section-row__badge">{jumps.length} jumps</span>
    {/if}
    {#if sourceRef}
      <span class="forge-section-row__source">{sourceRef.file}:{sourceRef.line}</span>
    {/if}
  </button>

  {#if expanded}
    <div class="forge-section-row__detail">
      {#each choices as choice}
        <button
          class="forge-section-row__choice"
          onclick={() => onNavigate(choice.span.file, choice.span.start_line)}
        >
          <span class="forge-section-row__choice-marker" class:forge-section-row__choice-marker--sticky={choice.sticky}>
            {choice.sticky ? '+' : '*'}
          </span>
          <span class="forge-section-row__choice-label">{choice.label}</span>
          {#if choice.condition_reads.length > 0}
            <span class="forge-section-row__choice-badge forge-section-row__choice-badge--conditional">conditional</span>
          {/if}
          {#if choice.sticky}
            <span class="forge-section-row__choice-badge forge-section-row__choice-badge--sticky">sticky</span>
          {/if}
        </button>

        {#if choice.condition_reads.length > 0 || choice.effect_writes.length > 0}
          <div class="forge-section-row__choice-details">
            {#each choice.condition_reads as readIdx}
              {@const read = factSet.reads[readIdx]}
              {#if read}
                <div class="forge-section-row__cond-line">
                  ? {read.entity_type}.{read.property} {read.operator} {read.value_literal}
                </div>
              {/if}
            {/each}
            {#each choice.effect_writes as writeIdx}
              {@const write = factSet.writes[writeIdx]}
              {#if write}
                <div class="forge-section-row__effect-line">
                  > {write.entity_type}.{write.property} {write.operator} {write.value_expr}
                </div>
              {/if}
            {/each}
          </div>
        {/if}
      {/each}

      {#each jumps as jump}
        <button
          class="forge-section-row__jump"
          onclick={() => onNavigate(jump.span.file, jump.span.start_line)}
        >
          <span class="forge-section-row__jump-arrow">\u2192</span>
          <span class="forge-section-row__jump-target">
            {jump.target.id ?? jump.target.kind}
          </span>
          <span class="forge-section-row__choice-badge">auto</span>
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .forge-section-row {
    border-bottom: 1px solid color-mix(in srgb, var(--forge-border-zone) 50%, transparent);
  }

  .forge-section-row__header {
    display: flex;
    align-items: center;
    gap: var(--forge-space-xs);
    width: 100%;
    height: 24px;
    padding: 0 var(--forge-space-sm);
    background: none;
    border: none;
    cursor: pointer;
    text-align: left;
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-xs);
    color: var(--forge-text-primary);
  }

  .forge-section-row__header:hover {
    background-color: var(--forge-table-row-hover);
  }

  .forge-section-row__toggle {
    flex-shrink: 0;
    width: 10px;
    font-size: 10px;
    color: var(--forge-text-muted);
  }

  .forge-section-row__id {
    color: var(--forge-text-primary);
    font-weight: 600;
    flex: 1;
  }

  .forge-section-row__badge {
    font-size: 10px;
    padding: 0 3px;
    border-radius: var(--forge-radius-sm);
    background-color: color-mix(in srgb, var(--forge-text-muted) 12%, transparent);
    color: var(--forge-text-muted);
    line-height: 14px;
    flex-shrink: 0;
  }

  .forge-section-row__source {
    color: var(--forge-text-muted);
    font-size: 10px;
    flex-shrink: 0;
  }

  .forge-section-row__detail {
    padding: var(--forge-space-xs) var(--forge-space-sm) var(--forge-space-sm);
    padding-left: calc(var(--forge-space-sm) + 16px);
  }

  .forge-section-row__choice {
    display: flex;
    align-items: center;
    gap: var(--forge-space-xs);
    width: 100%;
    height: 22px;
    padding: 0;
    background: none;
    border: none;
    cursor: pointer;
    text-align: left;
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-xs);
    color: var(--forge-text-secondary);
  }

  .forge-section-row__choice:hover {
    color: var(--forge-text-primary);
  }

  .forge-section-row__choice-marker {
    flex-shrink: 0;
    width: 10px;
    color: var(--forge-text-muted);
  }

  .forge-section-row__choice-marker--sticky {
    color: var(--forge-status-warning);
  }

  .forge-section-row__choice-label {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .forge-section-row__choice-badge {
    font-size: 9px;
    padding: 0 3px;
    border-radius: var(--forge-radius-sm);
    background-color: color-mix(in srgb, var(--forge-text-muted) 12%, transparent);
    color: var(--forge-text-muted);
    line-height: 14px;
    flex-shrink: 0;
  }

  .forge-section-row__choice-badge--conditional {
    background-color: color-mix(in srgb, var(--forge-status-warning) 15%, transparent);
    color: var(--forge-status-warning);
  }

  .forge-section-row__choice-badge--sticky {
    background-color: color-mix(in srgb, var(--forge-status-warning) 12%, transparent);
    color: var(--forge-status-warning);
  }

  .forge-section-row__choice-details {
    padding-left: 16px;
    margin-bottom: var(--forge-space-xs);
  }

  .forge-section-row__cond-line {
    font-family: var(--forge-font-family-mono);
    font-size: 10px;
    color: var(--forge-text-muted);
    height: 18px;
    display: flex;
    align-items: center;
  }

  .forge-section-row__effect-line {
    font-family: var(--forge-font-family-mono);
    font-size: 10px;
    color: var(--forge-status-success, #4ade80);
    height: 18px;
    display: flex;
    align-items: center;
  }

  .forge-section-row__jump {
    display: flex;
    align-items: center;
    gap: var(--forge-space-xs);
    width: 100%;
    height: 22px;
    padding: 0;
    background: none;
    border: none;
    cursor: pointer;
    text-align: left;
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-xs);
    color: var(--forge-text-secondary);
  }

  .forge-section-row__jump:hover {
    color: var(--forge-text-primary);
  }

  .forge-section-row__jump-arrow {
    color: var(--forge-status-success, #4ade80);
    flex-shrink: 0;
    width: 10px;
  }

  .forge-section-row__jump-target {
    color: var(--forge-status-success, #4ade80);
  }
</style>
