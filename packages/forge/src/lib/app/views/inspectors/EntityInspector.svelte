<script lang="ts">
  /**
   * EntityInspector — shows details of the selected entity.
   *
   * Subscribes to selection.primary. When an entity is selected, shows its
   * name, type badge, properties, contained items, and source link.
   */

  import { onMount, onDestroy } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { selectionContext, type SelectionState } from '$lib/framework/selection/SelectionContext';
  import { projectionRegistry } from '$lib/app/projections/ProjectionRegistry';
  import { navigationBroker } from '$lib/framework/navigation/NavigationBroker';
  import type { EntityRow } from '$lib/app/projections/entity-table';
  import type { PropertyRow } from '$lib/app/projections/property-table';
  import type { UrdWorld } from '$lib/app/compiler/types';
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

  let entityRow: EntityRow | null = $state(null);
  let properties: { key: string; value: string }[] = $state([]);
  let contains: string[] = $state([]);
  let entityType: string = $state('');

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
    if (state.items.length > 0 && state.items[0].kind === 'entity') {
      const item = state.items[0];
      selectedId = item.id;
      selectedLabel = item.label ?? item.id;
      selectedData = (item.data as Record<string, unknown>) ?? null;
      refreshData();
    } else {
      selectedId = null;
      selectedLabel = null;
      selectedData = null;
      entityRow = null;
      properties = [];
      contains = [];
      entityType = '';
    }
  }

  function refreshData(): void {
    if (!selectedId) {
      entityRow = null;
      properties = [];
      contains = [];
      entityType = '';
      return;
    }

    const entityRows = projectionRegistry.get<EntityRow[]>('urd.projection.entityTable') ?? [];
    entityRow = entityRows.find((r) => r.id === selectedId) ?? null;
    entityType = entityRow?.type ?? '';

    const propRows = projectionRegistry.get<PropertyRow[]>('urd.projection.propertyTable') ?? [];
    properties = propRows
      .filter((r) => r.subject === selectedId)
      .map((r) => ({ key: r.predicate, value: r.object }));

    const urdJson = projectionRegistry.get<UrdWorld>('urd.projection.urdJson');
    const entity = urdJson?.entities.find((e) => e.id === selectedId);
    contains = entity?.contains ?? [];
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
  title="Entity Inspector"
  emptyMessage="Select an entity to inspect"
  hasContent={selectedId !== null}
>
  {#if selectedId}
    <div class="forge-entity-inspector">
      <div class="forge-entity-inspector__header">
        <span class="forge-entity-inspector__kind">entity</span>
        <span class="forge-entity-inspector__name">{selectedLabel}</span>
        {#if entityType}
          <span class="forge-entity-inspector__kind">{entityType}</span>
        {/if}
      </div>

      {#if selectedData?.file}
        <button
          class="forge-entity-inspector__source-link"
          onclick={goToSource}
        >
          {selectedData.file}:{selectedData.line ?? '?'} →
        </button>
      {/if}

      {#if properties.length > 0}
        <InspectorSection title="Properties">
          <dl class="forge-entity-inspector__props">
            {#each properties as prop}
              <dt class="forge-entity-inspector__key">{prop.key}</dt>
              <dd class="forge-entity-inspector__value">{prop.value}</dd>
            {/each}
          </dl>
        </InspectorSection>
      {/if}

      {#if contains.length > 0}
        <InspectorSection title="Contains">
          <ul class="forge-entity-inspector__list">
            {#each contains as childId}
              <li class="forge-entity-inspector__list-item">{childId}</li>
            {/each}
          </ul>
        </InspectorSection>
      {/if}

      {#if entityRow}
        <InspectorSection title="Type Info">
          <dl class="forge-entity-inspector__props">
            <dt class="forge-entity-inspector__key">type</dt>
            <dd class="forge-entity-inspector__value">{entityRow.type || '(none)'}</dd>
            <dt class="forge-entity-inspector__key">properties</dt>
            <dd class="forge-entity-inspector__value">{entityRow.propertyCount}</dd>
            <dt class="forge-entity-inspector__key">reads</dt>
            <dd class="forge-entity-inspector__value">{entityRow.readCount}</dd>
            <dt class="forge-entity-inspector__key">writes</dt>
            <dd class="forge-entity-inspector__value">{entityRow.writeCount}</dd>
          </dl>
        </InspectorSection>
      {/if}
    </div>
  {/if}
</InspectorPanel>

<style>
  .forge-entity-inspector {
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-sm);
  }

  .forge-entity-inspector__header {
    display: flex;
    align-items: center;
    gap: var(--forge-space-sm);
    margin-bottom: var(--forge-space-md);
  }

  .forge-entity-inspector__kind {
    padding: 1px 6px;
    border-radius: var(--forge-radius-sm);
    background-color: var(--forge-bg-tertiary);
    color: var(--forge-text-muted);
    font-size: var(--forge-font-size-xs);
    text-transform: uppercase;
  }

  .forge-entity-inspector__name {
    font-weight: 600;
    color: var(--forge-text-primary);
    font-size: var(--forge-font-size-md);
  }

  .forge-entity-inspector__source-link {
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

  .forge-entity-inspector__source-link:hover {
    text-decoration: underline;
  }

  .forge-entity-inspector__props {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: var(--forge-space-xs) var(--forge-space-md);
    margin: 0;
  }

  .forge-entity-inspector__key {
    color: var(--forge-text-secondary);
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-xs);
  }

  .forge-entity-inspector__value {
    color: var(--forge-text-primary);
    margin: 0;
    word-break: break-word;
  }

  .forge-entity-inspector__list {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .forge-entity-inspector__list-item {
    padding: var(--forge-space-xs) 0;
    color: var(--forge-text-primary);
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-xs);
  }
</style>
