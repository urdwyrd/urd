<script lang="ts">
  /**
   * RuleInspector — shows details of the selected rule.
   *
   * Subscribes to selection.primary. When a rule is selected, shows its
   * conditions (reads), effects (writes), and source link.
   */

  import { onMount, onDestroy } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { selectionContext, type SelectionState } from '$lib/framework/selection/SelectionContext';
  import { projectionRegistry } from '$lib/app/projections/ProjectionRegistry';
  import { navigationBroker } from '$lib/framework/navigation/NavigationBroker';
  import type { FactSet, PropertyRead, PropertyWrite } from '$lib/app/compiler/types';
  import InspectorPanel from './_shared/InspectorPanel.svelte';
  import InspectorSection from './_shared/InspectorSection.svelte';

  interface Props {
    zoneId: string;
    zoneTypeId: string;
    state: null;
    onStateChange: (newState: unknown) => void;
  }

  let { zoneId, zoneTypeId, state: zoneState, onStateChange }: Props = $props();

  let selectedId: string | null = $state(null);
  let selectedLabel: string | null = $state(null);
  let selectedData: Record<string, unknown> | null = $state(null);

  let conditions: PropertyRead[] = $state([]);
  let effects: PropertyWrite[] = $state([]);
  let ruleFile: string = $state('');
  let ruleLine: number = $state(0);

  const unsubscribers: (() => void)[] = [];

  onMount(() => {
    updateFromSelection(selectionContext.state);

    unsubscribers.push(
      selectionContext.subscribe(updateFromSelection)
    );

    unsubscribers.push(
      bus.subscribe('compiler.completed', () => {
        if (selectedId) {
          refreshData();
        }
      })
    );
  });

  onDestroy(() => {
    for (const unsub of unsubscribers) unsub();
  });

  function updateFromSelection(state: SelectionState): void {
    if (state.items.length > 0 && state.items[0].kind === 'rule') {
      const item = state.items[0];
      selectedId = item.id;
      selectedLabel = item.label ?? item.id;
      selectedData = (item.data as Record<string, unknown>) ?? null;
      refreshData();
    } else {
      selectedId = null;
      selectedLabel = null;
      selectedData = null;
      conditions = [];
      effects = [];
      ruleFile = '';
      ruleLine = 0;
    }
  }

  function refreshData(): void {
    if (!selectedId) {
      conditions = [];
      effects = [];
      ruleFile = '';
      ruleLine = 0;
      return;
    }

    const factSet = projectionRegistry.get<FactSet>('urd.projection.factSet');
    if (!factSet) {
      conditions = [];
      effects = [];
      ruleFile = '';
      ruleLine = 0;
      return;
    }

    const rule = factSet.rules.find((r) => r.rule_id === selectedId);
    if (!rule) {
      conditions = [];
      effects = [];
      ruleFile = '';
      ruleLine = 0;
      return;
    }

    ruleFile = rule.span.file;
    ruleLine = rule.span.start_line;

    conditions = rule.condition_reads
      .map((idx) => factSet.reads[idx])
      .filter((r): r is PropertyRead => r !== undefined);

    effects = rule.effect_writes
      .map((idx) => factSet.writes[idx])
      .filter((w): w is PropertyWrite => w !== undefined);
  }

  function goToSource(): void {
    const file = selectedData?.file as string | undefined ?? ruleFile;
    const line = selectedData?.line as number | undefined ?? ruleLine;
    if (file) {
      navigationBroker.navigate({
        targetViewId: 'urd.codeEditor',
        params: { path: file, line },
      });
    }
  }
</script>

<InspectorPanel
  title="Rule Inspector"
  emptyMessage="Select a rule to inspect"
  hasContent={selectedId !== null}
>
  {#if selectedId}
    <div class="forge-rule-inspector">
      <div class="forge-rule-inspector__header">
        <span class="forge-rule-inspector__kind">rule</span>
        <span class="forge-rule-inspector__name">{selectedLabel}</span>
      </div>

      {#if ruleFile || selectedData?.file}
        <button
          class="forge-rule-inspector__source-link"
          onclick={goToSource}
        >
          {selectedData?.file ?? ruleFile}:{selectedData?.line ?? ruleLine ?? '?'} →
        </button>
      {/if}

      {#if conditions.length > 0}
        <InspectorSection title="Conditions ({conditions.length})">
          <dl class="forge-rule-inspector__props">
            {#each conditions as cond}
              <dt class="forge-rule-inspector__key">{cond.entity_type}.{cond.property}</dt>
              <dd class="forge-rule-inspector__value">{cond.operator} {cond.value_literal}</dd>
            {/each}
          </dl>
        </InspectorSection>
      {/if}

      {#if effects.length > 0}
        <InspectorSection title="Effects ({effects.length})">
          <dl class="forge-rule-inspector__props">
            {#each effects as eff}
              <dt class="forge-rule-inspector__key">{eff.entity_type}.{eff.property}</dt>
              <dd class="forge-rule-inspector__value">{eff.operator} {eff.value_expr}</dd>
            {/each}
          </dl>
        </InspectorSection>
      {/if}
    </div>
  {/if}
</InspectorPanel>

<style>
  .forge-rule-inspector {
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-sm);
  }

  .forge-rule-inspector__header {
    display: flex;
    align-items: center;
    gap: var(--forge-space-sm);
    margin-bottom: var(--forge-space-md);
  }

  .forge-rule-inspector__kind {
    padding: 1px 6px;
    border-radius: var(--forge-radius-sm);
    background-color: var(--forge-bg-tertiary);
    color: var(--forge-text-muted);
    font-size: var(--forge-font-size-xs);
    text-transform: uppercase;
  }

  .forge-rule-inspector__name {
    font-weight: 600;
    color: var(--forge-text-primary);
    font-size: var(--forge-font-size-md);
  }

  .forge-rule-inspector__source-link {
    display: inline-block;
    margin-bottom: var(--forge-space-md);
    padding: 0;
    background: none;
    border: none;
    color: var(--forge-accent-primary);
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-xs);
    cursor: pointer;
    text-decoration: none;
  }

  .forge-rule-inspector__source-link:hover {
    text-decoration: underline;
  }

  .forge-rule-inspector__props {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: var(--forge-space-xs) var(--forge-space-md);
    margin: 0;
  }

  .forge-rule-inspector__key {
    color: var(--forge-text-secondary);
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-xs);
  }

  .forge-rule-inspector__value {
    color: var(--forge-text-primary);
    margin: 0;
    word-break: break-word;
  }
</style>
