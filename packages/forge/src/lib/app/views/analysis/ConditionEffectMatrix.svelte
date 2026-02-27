<script lang="ts">
  /**
   * ConditionEffectMatrix — on-demand cross-reference of rules vs properties.
   *
   * Shows a "Compute Matrix" button when idle. On click, reads factSet.rules
   * and cross-references with factSet.reads/writes to build a grid showing
   * which properties each rule reads (R), writes (W), or both (RW).
   */

  import { onMount, onDestroy } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { projectionRegistry } from '$lib/app/projections/ProjectionRegistry';
  import type { FactSet } from '$lib/app/compiler/types';

  interface Props {
    zoneId: string;
    zoneTypeId: string;
    state: null;
    onStateChange: (newState: unknown) => void;
  }

  let { zoneId, zoneTypeId, state: zoneState, onStateChange }: Props = $props();

  type ComputeState = 'idle' | 'computing' | 'done';

  interface MatrixCell {
    reads: boolean;
    writes: boolean;
  }

  let computeState: ComputeState = $state('idle');
  let ruleIds: string[] = $state([]);
  let propertyKeys: string[] = $state([]);
  let cells: MatrixCell[][] = $state([]);
  let stale = $state(false);

  const unsubscribers: (() => void)[] = [];

  onMount(() => {
    unsubscribers.push(bus.subscribe('compiler.completed', () => {
      if (computeState === 'done') stale = true;
    }));
  });

  onDestroy(() => {
    for (const unsub of unsubscribers) unsub();
  });

  function computeMatrix(): void {
    computeState = 'computing';
    stale = false;

    const factSetData = projectionRegistry.get<FactSet>('urd.projection.factSet');
    if (!factSetData) {
      computeState = 'idle';
      return;
    }

    const { rules, reads, writes } = factSetData;

    // Collect unique properties involved in rules
    const propSet = new Set<string>();
    const ruleReads = new Map<string, Set<string>>();
    const ruleWrites = new Map<string, Set<string>>();

    for (const rule of rules) {
      const ruleId = rule.rule_id;

      const readProps = new Set<string>();
      for (const readIdx of rule.condition_reads) {
        const read = reads[readIdx];
        if (read) {
          const key = `${read.entity_type}.${read.property}`;
          readProps.add(key);
          propSet.add(key);
        }
      }
      ruleReads.set(ruleId, readProps);

      const writeProps = new Set<string>();
      for (const writeIdx of rule.effect_writes) {
        const write = writes[writeIdx];
        if (write) {
          const key = `${write.entity_type}.${write.property}`;
          writeProps.add(key);
          propSet.add(key);
        }
      }
      ruleWrites.set(ruleId, writeProps);
    }

    const sortedRuleIds = rules.map((r) => r.rule_id);
    const sortedProps = [...propSet].sort();

    // Build cell matrix
    const result: MatrixCell[][] = [];
    for (const ruleId of sortedRuleIds) {
      const row: MatrixCell[] = [];
      const rReads = ruleReads.get(ruleId) ?? new Set();
      const rWrites = ruleWrites.get(ruleId) ?? new Set();

      for (const prop of sortedProps) {
        row.push({
          reads: rReads.has(prop),
          writes: rWrites.has(prop),
        });
      }
      result.push(row);
    }

    ruleIds = sortedRuleIds;
    propertyKeys = sortedProps;
    cells = result;
    computeState = 'done';
  }

  function cellLabel(cell: MatrixCell): string {
    if (cell.reads && cell.writes) return 'RW';
    if (cell.reads) return 'R';
    if (cell.writes) return 'W';
    return '';
  }
</script>

<div class="forge-analysis-condeffect">
  {#if computeState === 'idle'}
    <div class="forge-analysis-condeffect__prompt">
      <p class="forge-analysis-condeffect__description">
        Cross-reference rules with properties to see which properties each rule reads and writes.
      </p>
      <button
        class="forge-analysis-condeffect__compute-btn"
        onclick={computeMatrix}
      >
        Compute Matrix
      </button>
    </div>
  {:else if computeState === 'computing'}
    <div class="forge-analysis-condeffect__computing">
      Computing condition/effect matrix...
    </div>
  {:else}
    <div class="forge-analysis-condeffect__results">
      {#if stale}
        <div class="forge-analysis-condeffect__stale-banner">
          Data may be stale.
          <button
            class="forge-analysis-condeffect__recompute-btn"
            onclick={computeMatrix}
          >
            Recompute
          </button>
        </div>
      {/if}

      <div class="forge-analysis-condeffect__summary">
        {ruleIds.length} rules x {propertyKeys.length} properties
      </div>

      {#if ruleIds.length === 0}
        <div class="forge-analysis-condeffect__empty">
          No rules found in the world
        </div>
      {:else}
        <div class="forge-analysis-condeffect__matrix-wrapper">
          <table class="forge-analysis-condeffect__table">
            <thead>
              <tr>
                <th class="forge-analysis-condeffect__corner">Rule</th>
                {#each propertyKeys as prop}
                  <th class="forge-analysis-condeffect__col-header" title={prop}>
                    {prop.length > 12 ? prop.slice(0, 11) + '...' : prop}
                  </th>
                {/each}
              </tr>
            </thead>
            <tbody>
              {#each ruleIds as ruleId, i}
                <tr>
                  <td class="forge-analysis-condeffect__row-header" title={ruleId}>
                    {ruleId.length > 16 ? ruleId.slice(0, 15) + '...' : ruleId}
                  </td>
                  {#each cells[i] as cell}
                    <td
                      class="forge-analysis-condeffect__cell"
                      class:forge-analysis-condeffect__cell--read={cell.reads && !cell.writes}
                      class:forge-analysis-condeffect__cell--write={cell.writes && !cell.reads}
                      class:forge-analysis-condeffect__cell--rw={cell.reads && cell.writes}
                    >
                      {cellLabel(cell)}
                    </td>
                  {/each}
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .forge-analysis-condeffect {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    overflow: auto;
    font-family: var(--forge-font-family-ui);
  }

  .forge-analysis-condeffect__prompt {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    gap: var(--forge-space-lg);
    padding: var(--forge-space-xl);
  }

  .forge-analysis-condeffect__description {
    color: var(--forge-text-muted);
    font-size: var(--forge-font-size-md);
    text-align: center;
    max-width: 400px;
  }

  .forge-analysis-condeffect__compute-btn,
  .forge-analysis-condeffect__recompute-btn {
    padding: var(--forge-space-sm) var(--forge-space-lg);
    background-color: var(--forge-bg-secondary);
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-md);
    color: var(--forge-text-primary);
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-sm);
    cursor: pointer;
  }

  .forge-analysis-condeffect__compute-btn:hover,
  .forge-analysis-condeffect__recompute-btn:hover {
    background-color: var(--forge-table-row-hover);
  }

  .forge-analysis-condeffect__computing {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    color: var(--forge-text-muted);
    font-size: var(--forge-font-size-md);
  }

  .forge-analysis-condeffect__results {
    display: flex;
    flex-direction: column;
    padding: var(--forge-space-md);
    gap: var(--forge-space-md);
  }

  .forge-analysis-condeffect__stale-banner {
    display: flex;
    align-items: center;
    gap: var(--forge-space-sm);
    padding: var(--forge-space-sm) var(--forge-space-md);
    background-color: var(--forge-bg-secondary);
    border: 1px solid var(--forge-status-warning);
    border-radius: var(--forge-radius-sm);
    color: var(--forge-status-warning);
    font-size: var(--forge-font-size-sm);
  }

  .forge-analysis-condeffect__summary {
    font-size: var(--forge-font-size-sm);
    color: var(--forge-text-secondary);
  }

  .forge-analysis-condeffect__empty {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--forge-space-xl);
    color: var(--forge-text-muted);
    font-size: var(--forge-font-size-md);
  }

  .forge-analysis-condeffect__matrix-wrapper {
    overflow: auto;
  }

  .forge-analysis-condeffect__table {
    border-collapse: collapse;
    font-size: var(--forge-font-size-xs);
  }

  .forge-analysis-condeffect__corner {
    padding: var(--forge-space-xs) var(--forge-space-sm);
    text-align: left;
    color: var(--forge-text-secondary);
    font-weight: 600;
  }

  .forge-analysis-condeffect__col-header {
    writing-mode: vertical-lr;
    text-orientation: mixed;
    padding: var(--forge-space-xs);
    color: var(--forge-text-secondary);
    font-weight: 500;
    font-family: var(--forge-font-family-mono);
    max-width: 28px;
    overflow: hidden;
  }

  .forge-analysis-condeffect__row-header {
    padding: var(--forge-space-xs) var(--forge-space-sm);
    color: var(--forge-text-secondary);
    font-weight: 500;
    font-family: var(--forge-font-family-mono);
    text-align: right;
    white-space: nowrap;
  }

  .forge-analysis-condeffect__cell {
    width: 28px;
    height: 24px;
    min-width: 28px;
    text-align: center;
    border: 1px solid var(--forge-border-zone);
    font-family: var(--forge-font-family-mono);
    font-weight: 600;
    font-size: 9px;
    color: var(--forge-text-primary);
  }

  .forge-analysis-condeffect__cell--read {
    background-color: rgba(100, 180, 255, 0.25);
  }

  .forge-analysis-condeffect__cell--write {
    background-color: rgba(255, 160, 100, 0.25);
  }

  .forge-analysis-condeffect__cell--rw {
    background-color: rgba(180, 120, 255, 0.25);
  }
</style>
