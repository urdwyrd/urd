<script lang="ts">
  /**
   * RuleRow — expandable rule detail row within RulesEngine.
   * L0: rule ID, condition count, effect count, source ref.
   * L1: conditions, effects, selector.
   */

  import type { UrdRule, RuleFact, FactSet } from '$lib/app/compiler/types';

  interface Props {
    ruleId: string;
    rule: UrdRule;
    ruleFact: RuleFact | undefined;
    factSet: FactSet;
    expanded: boolean;
    onToggle: (key: string) => void;
    onNavigate: (path: string, line: number) => void;
  }

  let { ruleId, rule, ruleFact, factSet, expanded, onToggle, onNavigate }: Props = $props();

  let conditionCount = $derived(ruleFact?.condition_reads.length ?? rule.conditions?.length ?? 0);
  let effectCount = $derived(ruleFact?.effect_writes.length ?? rule.effects.length);

  function handleToggle(): void {
    onToggle(`rule:${ruleId}`);
  }
</script>

<div class="forge-rule-row">
  <div class="forge-rule-row__header" role="button" tabindex="0" onclick={handleToggle} onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); handleToggle(); } }}>
    <span class="forge-rule-row__toggle">{expanded ? '\u25BE' : '\u25B8'}</span>
    <span class="forge-rule-row__id">{ruleId}</span>
    <span class="forge-rule-row__badge">{conditionCount} conditions</span>
    <span class="forge-rule-row__badge">{effectCount} effects</span>
    {#if ruleFact?.span}
      <button
        class="forge-rule-row__source"
        onclick={(e) => { e.stopPropagation(); if (ruleFact?.span) onNavigate(ruleFact.span.file, ruleFact.span.start_line); }}
      >
        {ruleFact.span.file}:{ruleFact.span.start_line}
      </button>
    {/if}
  </div>

  {#if expanded}
    <div class="forge-rule-row__detail">
      <!-- Conditions from FactSet reads -->
      {#if ruleFact && ruleFact.condition_reads.length > 0}
        <div class="forge-rule-row__sub-header">CONDITIONS</div>
        {#each ruleFact.condition_reads as readIdx}
          {@const read = factSet.reads[readIdx]}
          {#if read}
            <div class="forge-rule-row__cond-line">
              ? {read.entity_type}.{read.property} {read.operator} {read.value_literal}
            </div>
          {/if}
        {/each}
      {:else if rule.conditions && rule.conditions.length > 0}
        <div class="forge-rule-row__sub-header">CONDITIONS</div>
        {#each rule.conditions as cond}
          <div class="forge-rule-row__cond-line">? {cond}</div>
        {/each}
      {/if}

      <!-- Effects from FactSet writes -->
      {#if ruleFact && ruleFact.effect_writes.length > 0}
        <div class="forge-rule-row__sub-header">EFFECTS</div>
        {#each ruleFact.effect_writes as writeIdx}
          {@const write = factSet.writes[writeIdx]}
          {#if write}
            <div class="forge-rule-row__effect-line">
              > {write.entity_type}.{write.property} {write.operator} {write.value_expr}
            </div>
          {/if}
        {/each}
      {:else if rule.effects.length > 0}
        <div class="forge-rule-row__sub-header">EFFECTS</div>
        {#each rule.effects as effect}
          <div class="forge-rule-row__effect-line">
            > {effect.set ? `${effect.set} = ${effect.to}` : effect.move ? `move ${effect.move}` : effect.destroy ? `destroy ${effect.destroy}` : effect.reveal ? `reveal ${effect.reveal}` : '?'}
          </div>
        {/each}
      {/if}

      <!-- Selector -->
      {#if rule.select}
        <div class="forge-rule-row__sub-header">SELECTOR</div>
        <div class="forge-rule-row__selector">
          {rule.actor ?? '?'} selects {rule.select.as} from [{rule.select.from.join(', ')}]
        </div>
        {#if rule.select.where}
          {#each rule.select.where as whereCond}
            <div class="forge-rule-row__cond-line">where {whereCond}</div>
          {/each}
        {/if}
      {/if}

      <!-- Trigger -->
      <div class="forge-rule-row__sub-header">TRIGGER</div>
      <div class="forge-rule-row__trigger">{rule.trigger}</div>
    </div>
  {/if}
</div>

<style>
  .forge-rule-row {
    border-bottom: 1px solid color-mix(in srgb, var(--forge-border-zone) 50%, transparent);
  }

  .forge-rule-row__header {
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

  .forge-rule-row__header:hover {
    background-color: var(--forge-table-row-hover);
  }

  .forge-rule-row__toggle {
    flex-shrink: 0;
    width: 10px;
    font-size: 10px;
    color: var(--forge-text-muted);
  }

  .forge-rule-row__id {
    color: var(--forge-text-primary);
    font-weight: 600;
    flex: 1;
  }

  .forge-rule-row__badge {
    font-size: 10px;
    padding: 0 3px;
    border-radius: var(--forge-radius-sm);
    background-color: color-mix(in srgb, var(--forge-text-muted) 12%, transparent);
    color: var(--forge-text-muted);
    line-height: 14px;
    flex-shrink: 0;
  }

  .forge-rule-row__source {
    background: none;
    border: none;
    padding: 0;
    color: var(--forge-text-muted);
    font-family: var(--forge-font-family-mono);
    font-size: 10px;
    cursor: pointer;
    flex-shrink: 0;
  }

  .forge-rule-row__source:hover {
    text-decoration: underline;
  }

  .forge-rule-row__detail {
    padding: var(--forge-space-xs) var(--forge-space-sm) var(--forge-space-sm);
    padding-left: calc(var(--forge-space-sm) + 16px);
  }

  .forge-rule-row__sub-header {
    font-family: var(--forge-font-family-ui);
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.03em;
    color: var(--forge-text-muted);
    margin-top: var(--forge-space-xs);
    margin-bottom: 2px;
  }

  .forge-rule-row__cond-line {
    font-family: var(--forge-font-family-mono);
    font-size: 10px;
    color: var(--forge-text-muted);
    height: 18px;
    display: flex;
    align-items: center;
  }

  .forge-rule-row__effect-line {
    font-family: var(--forge-font-family-mono);
    font-size: 10px;
    color: var(--forge-status-success, #4ade80);
    height: 18px;
    display: flex;
    align-items: center;
  }

  .forge-rule-row__selector {
    font-family: var(--forge-font-family-mono);
    font-size: 10px;
    color: var(--forge-text-secondary);
    height: 18px;
    display: flex;
    align-items: center;
  }

  .forge-rule-row__trigger {
    font-family: var(--forge-font-family-mono);
    font-size: 10px;
    color: var(--forge-text-secondary);
    height: 18px;
    display: flex;
    align-items: center;
  }
</style>
