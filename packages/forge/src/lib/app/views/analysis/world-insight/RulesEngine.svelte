<script lang="ts">
  /**
   * RulesEngine — rules list (Section 6).
   * Iterates world.rules cross-referenced with factSet.rules.
   */

  import type { UrdWorld, FactSet, RuleFact } from '$lib/app/compiler/types';
  import RuleRow from './RuleRow.svelte';

  interface Props {
    world: UrdWorld;
    factSet: FactSet | null;
    expandedRows: Record<string, boolean>;
    onToggleRow: (key: string) => void;
    onNavigate: (path: string, line: number) => void;
    filterText: string;
  }

  let { world, factSet, expandedRows, onToggleRow, onNavigate, filterText }: Props = $props();

  // Build rule ID → RuleFact map
  let ruleFactMap = $derived.by(() => {
    const map = new Map<string, RuleFact>();
    if (factSet) {
      for (const rf of factSet.rules) {
        map.set(rf.rule_id, rf);
      }
    }
    return map;
  });

  let ruleEntries = $derived.by(() => {
    if (!world.rules) return [];
    const q = filterText.toLowerCase();
    return Object.entries(world.rules)
      .filter(([ruleId]) => !q || ruleId.toLowerCase().includes(q))
      .sort((a, b) => a[0].localeCompare(b[0]));
  });
</script>

<div class="forge-rules-engine">
  {#if ruleEntries.length === 0}
    <div class="forge-rules-engine__empty">No rules</div>
  {:else if factSet}
    {#each ruleEntries as [ruleId, rule]}
      <RuleRow
        {ruleId}
        {rule}
        ruleFact={ruleFactMap.get(ruleId)}
        {factSet}
        expanded={expandedRows[`rule:${ruleId}`] ?? false}
        onToggle={onToggleRow}
        {onNavigate}
      />
    {/each}
  {/if}
</div>

<style>
  .forge-rules-engine {
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-xs);
  }

  .forge-rules-engine__empty {
    padding: var(--forge-space-md);
    color: var(--forge-text-muted);
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-sm);
    text-align: center;
  }
</style>
