<script lang="ts">
  /**
   * SectionInspector — shows details of the selected narrative section.
   *
   * Subscribes to selection.primary. When a section is selected, shows its
   * choices, outgoing jumps, incoming references, and source link.
   */

  import { onMount, onDestroy } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { selectionContext, type SelectionState } from '$lib/framework/selection/SelectionContext';
  import { projectionRegistry } from '$lib/app/projections/ProjectionRegistry';
  import { navigationBroker } from '$lib/framework/navigation/NavigationBroker';
  import type { FactSet, ChoiceFact, JumpEdge } from '$lib/app/compiler/types';
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

  let choices: ChoiceFact[] = $state([]);
  let jumpsFrom: JumpEdge[] = $state([]);
  let referencedBy: JumpEdge[] = $state([]);

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
    if (state.items.length > 0 && state.items[0].kind === 'section') {
      const item = state.items[0];
      selectedId = item.id;
      selectedLabel = item.label ?? item.id;
      selectedData = (item.data as Record<string, unknown>) ?? null;
      refreshData();
    } else {
      selectedId = null;
      selectedLabel = null;
      selectedData = null;
      choices = [];
      jumpsFrom = [];
      referencedBy = [];
    }
  }

  function refreshData(): void {
    if (!selectedId) {
      choices = [];
      jumpsFrom = [];
      referencedBy = [];
      return;
    }

    const factSet = projectionRegistry.get<FactSet>('urd.projection.factSet');
    if (!factSet) {
      choices = [];
      jumpsFrom = [];
      referencedBy = [];
      return;
    }

    choices = factSet.choices.filter((c) => c.section === selectedId);
    jumpsFrom = factSet.jumps.filter((j) => j.from_section === selectedId);
    referencedBy = factSet.jumps.filter(
      (j) => j.target.id === selectedId && j.from_section !== selectedId
    );
  }

  function goToSource(): void {
    const file = selectedData?.file as string | undefined;
    const line = selectedData?.line as number | undefined;
    if (file) {
      navigationBroker.navigate({
        targetViewId: 'urd.codeEditor',
        params: { path: file, line },
      });
    }
  }
</script>

<InspectorPanel
  title="Section Inspector"
  emptyMessage="Select a section to inspect"
  hasContent={selectedId !== null}
>
  {#if selectedId}
    <div class="forge-section-inspector">
      <div class="forge-section-inspector__header">
        <span class="forge-section-inspector__kind">section</span>
        <span class="forge-section-inspector__name">{selectedLabel}</span>
      </div>

      {#if selectedData?.file}
        <button
          class="forge-section-inspector__source-link"
          onclick={goToSource}
        >
          {selectedData.file}:{selectedData.line ?? '?'} →
        </button>
      {/if}

      {#if choices.length > 0}
        <InspectorSection title="Choices ({choices.length})">
          <dl class="forge-section-inspector__props">
            {#each choices as choice}
              <dt class="forge-section-inspector__key">{choice.choice_id}</dt>
              <dd class="forge-section-inspector__value">
                {choice.label}{#if choice.sticky} (sticky){/if}
              </dd>
            {/each}
          </dl>
        </InspectorSection>
      {/if}

      {#if jumpsFrom.length > 0}
        <InspectorSection title="Jumps From ({jumpsFrom.length})">
          <dl class="forge-section-inspector__props">
            {#each jumpsFrom as jump}
              <dt class="forge-section-inspector__key">{jump.target.kind}</dt>
              <dd class="forge-section-inspector__value">{jump.target.id ?? '(end)'}</dd>
            {/each}
          </dl>
        </InspectorSection>
      {/if}

      {#if referencedBy.length > 0}
        <InspectorSection title="Referenced By ({referencedBy.length})">
          <ul class="forge-section-inspector__list">
            {#each referencedBy as ref}
              <li class="forge-section-inspector__list-item">{ref.from_section}</li>
            {/each}
          </ul>
        </InspectorSection>
      {/if}
    </div>
  {/if}
</InspectorPanel>

<style>
  .forge-section-inspector {
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-sm);
  }

  .forge-section-inspector__header {
    display: flex;
    align-items: center;
    gap: var(--forge-space-sm);
    margin-bottom: var(--forge-space-md);
  }

  .forge-section-inspector__kind {
    padding: 1px 6px;
    border-radius: var(--forge-radius-sm);
    background-color: var(--forge-bg-tertiary);
    color: var(--forge-text-muted);
    font-size: var(--forge-font-size-xs);
    text-transform: uppercase;
  }

  .forge-section-inspector__name {
    font-weight: 600;
    color: var(--forge-text-primary);
    font-size: var(--forge-font-size-md);
  }

  .forge-section-inspector__source-link {
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

  .forge-section-inspector__source-link:hover {
    text-decoration: underline;
  }

  .forge-section-inspector__props {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: var(--forge-space-xs) var(--forge-space-md);
    margin: 0;
  }

  .forge-section-inspector__key {
    color: var(--forge-text-secondary);
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-xs);
  }

  .forge-section-inspector__value {
    color: var(--forge-text-primary);
    margin: 0;
    word-break: break-word;
  }

  .forge-section-inspector__list {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .forge-section-inspector__list-item {
    padding: var(--forge-space-xs) 0;
    color: var(--forge-text-primary);
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-xs);
  }
</style>
