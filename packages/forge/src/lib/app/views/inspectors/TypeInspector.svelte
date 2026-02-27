<script lang="ts">
  /**
   * TypeInspector — shows details of the selected type definition.
   *
   * Subscribes to selection.primary. When a type is selected, shows its
   * traits, property schema, instances, and source link.
   */

  import { onMount, onDestroy } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { selectionContext, type SelectionState } from '$lib/framework/selection/SelectionContext';
  import { projectionRegistry } from '$lib/app/projections/ProjectionRegistry';
  import { navigationBroker } from '$lib/framework/navigation/NavigationBroker';
  import type { EntityRow } from '$lib/app/projections/entity-table';
  import type { UrdWorld, UrdTypeDef, UrdPropertyDef } from '$lib/app/compiler/types';
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

  let typeDef: UrdTypeDef | null = $state(null);
  let traits: string[] = $state([]);
  let propertySchema: { name: string; type: string; defaultVal: string }[] = $state([]);
  let instances: EntityRow[] = $state([]);

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
    if (state.items.length > 0 && state.items[0].kind === 'type') {
      const item = state.items[0];
      selectedId = item.id;
      selectedLabel = item.label ?? item.id;
      selectedData = (item.data as Record<string, unknown>) ?? null;
      refreshData();
    } else {
      selectedId = null;
      selectedLabel = null;
      selectedData = null;
      typeDef = null;
      traits = [];
      propertySchema = [];
      instances = [];
    }
  }

  function refreshData(): void {
    if (!selectedId) {
      typeDef = null;
      traits = [];
      propertySchema = [];
      instances = [];
      return;
    }

    const urdJson = projectionRegistry.get<UrdWorld>('urd.projection.urdJson');
    const typeName = selectedLabel ?? selectedId;

    if (urdJson?.types) {
      typeDef = urdJson.types[selectedId] ?? urdJson.types[typeName] ?? null;
    } else {
      typeDef = null;
    }

    traits = typeDef?.traits ?? [];

    if (typeDef?.properties) {
      propertySchema = Object.entries(typeDef.properties).map(([name, def]: [string, UrdPropertyDef]) => ({
        name,
        type: def.type,
        defaultVal: def.default !== undefined ? String(def.default) : '',
      }));
    } else {
      propertySchema = [];
    }

    const entityRows = projectionRegistry.get<EntityRow[]>('urd.projection.entityTable') ?? [];
    instances = entityRows.filter((r) => r.type === typeName || r.type === selectedId);
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
  title="Type Inspector"
  emptyMessage="Select a type to inspect"
  hasContent={selectedId !== null}
>
  {#if selectedId}
    <div class="forge-type-inspector">
      <div class="forge-type-inspector__header">
        <span class="forge-type-inspector__kind">type</span>
        <span class="forge-type-inspector__name">{selectedLabel}</span>
      </div>

      {#if selectedData?.file}
        <button
          class="forge-type-inspector__source-link"
          onclick={goToSource}
        >
          {selectedData.file}:{selectedData.line ?? '?'} →
        </button>
      {/if}

      {#if traits.length > 0}
        <InspectorSection title="Traits">
          <ul class="forge-type-inspector__list">
            {#each traits as trait}
              <li class="forge-type-inspector__list-item">{trait}</li>
            {/each}
          </ul>
        </InspectorSection>
      {/if}

      {#if propertySchema.length > 0}
        <InspectorSection title="Property Schema">
          <dl class="forge-type-inspector__props">
            {#each propertySchema as prop}
              <dt class="forge-type-inspector__key">{prop.name}</dt>
              <dd class="forge-type-inspector__value">
                {prop.type}{#if prop.defaultVal} = {prop.defaultVal}{/if}
              </dd>
            {/each}
          </dl>
        </InspectorSection>
      {/if}

      {#if instances.length > 0}
        <InspectorSection title="Instances ({instances.length})">
          <ul class="forge-type-inspector__list">
            {#each instances as entity}
              <li class="forge-type-inspector__list-item">{entity.name}</li>
            {/each}
          </ul>
        </InspectorSection>
      {/if}
    </div>
  {/if}
</InspectorPanel>

<style>
  .forge-type-inspector {
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-sm);
  }

  .forge-type-inspector__header {
    display: flex;
    align-items: center;
    gap: var(--forge-space-sm);
    margin-bottom: var(--forge-space-md);
  }

  .forge-type-inspector__kind {
    padding: 1px 6px;
    border-radius: var(--forge-radius-sm);
    background-color: var(--forge-bg-tertiary);
    color: var(--forge-text-muted);
    font-size: var(--forge-font-size-xs);
    text-transform: uppercase;
  }

  .forge-type-inspector__name {
    font-weight: 600;
    color: var(--forge-text-primary);
    font-size: var(--forge-font-size-md);
  }

  .forge-type-inspector__source-link {
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

  .forge-type-inspector__source-link:hover {
    text-decoration: underline;
  }

  .forge-type-inspector__props {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: var(--forge-space-xs) var(--forge-space-md);
    margin: 0;
  }

  .forge-type-inspector__key {
    color: var(--forge-text-secondary);
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-xs);
  }

  .forge-type-inspector__value {
    color: var(--forge-text-primary);
    margin: 0;
    word-break: break-word;
  }

  .forge-type-inspector__list {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .forge-type-inspector__list-item {
    padding: var(--forge-space-xs) 0;
    color: var(--forge-text-primary);
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-xs);
  }
</style>
