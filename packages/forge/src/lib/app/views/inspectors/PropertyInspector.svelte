<script lang="ts">
  /**
   * PropertyInspector — shows details of the selected entity/location.
   *
   * Subscribes to selection.primary. When an entity is selected, shows its
   * name, type, all properties as key-value pairs, and source link.
   */

  import { onMount, onDestroy } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { selectionContext, type SelectionState } from '$lib/framework/selection/SelectionContext';
  import { projectionRegistry } from '$lib/app/projections/ProjectionRegistry';
  import { navigationBroker } from '$lib/framework/navigation/NavigationBroker';
  import type { EntityRow } from '$lib/app/projections/entity-table';
  import type { PropertyRow } from '$lib/app/projections/property-table';
  import type { LocationRow } from '$lib/app/projections/location-table';
  import InspectorPanel from './_shared/InspectorPanel.svelte';
  import InspectorSection from './_shared/InspectorSection.svelte';

  interface Props {
    zoneId: string;
    zoneTypeId: string;
    state: null;
    onStateChange: (newState: unknown) => void;
  }

  let { zoneId, zoneTypeId, state: zoneState, onStateChange }: Props = $props();

  let selectedKind: string | null = $state(null);
  let selectedId: string | null = $state(null);
  let selectedLabel: string | null = $state(null);
  let selectedData: Record<string, unknown> | null = $state(null);
  let properties: { key: string; value: string }[] = $state([]);

  const unsubscribers: (() => void)[] = [];

  onMount(() => {
    // Read initial selection
    updateFromSelection(selectionContext.state);

    unsubscribers.push(
      selectionContext.subscribe(updateFromSelection)
    );

    unsubscribers.push(
      bus.subscribe('compiler.completed', () => {
        // Refresh after recompile
        if (selectedId) {
          refreshProperties();
        }
      })
    );
  });

  onDestroy(() => {
    for (const unsub of unsubscribers) unsub();
  });

  function updateFromSelection(state: SelectionState): void {
    if (state.items.length > 0) {
      const item = state.items[0];
      selectedKind = item.kind;
      selectedId = item.id;
      selectedLabel = item.label ?? item.id;
      selectedData = (item.data as Record<string, unknown>) ?? null;
      refreshProperties();
    } else {
      selectedKind = null;
      selectedId = null;
      selectedLabel = null;
      selectedData = null;
      properties = [];
    }
  }

  function refreshProperties(): void {
    if (!selectedId) {
      properties = [];
      return;
    }

    if (selectedKind === 'entity') {
      const propRows = projectionRegistry.get<PropertyRow[]>('urd.projection.propertyTable') ?? [];
      properties = propRows
        .filter((r) => r.subject === selectedId)
        .map((r) => ({ key: r.predicate, value: r.object }));
    } else if (selectedKind === 'location') {
      const locations = projectionRegistry.get<LocationRow[]>('urd.projection.locationTable') ?? [];
      const loc = locations.find((l) => l.id === selectedId);
      if (loc) {
        properties = [
          { key: 'description', value: loc.description },
          { key: 'exits', value: String(loc.exitCount) },
        ];
      } else {
        properties = [];
      }
    } else {
      properties = [];
    }
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
  title="Inspector"
  emptyMessage="Select an entity or location to inspect"
  hasContent={selectedId !== null}
>
  {#if selectedId}
    <div class="forge-property-inspector">
      <div class="forge-property-inspector__header">
        <span class="forge-property-inspector__kind">{selectedKind}</span>
        <span class="forge-property-inspector__name">{selectedLabel}</span>
      </div>

      {#if selectedData?.file}
        <button
          class="forge-property-inspector__source-link"
          onclick={goToSource}
        >
          {selectedData.file}:{selectedData.line ?? '?'} \u2192
        </button>
      {/if}

      {#if properties.length > 0}
        <InspectorSection title="Properties">
          <dl class="forge-property-inspector__props">
            {#each properties as prop}
              <dt class="forge-property-inspector__key">{prop.key}</dt>
              <dd class="forge-property-inspector__value">{prop.value}</dd>
            {/each}
          </dl>
        </InspectorSection>
      {/if}
    </div>
  {/if}
</InspectorPanel>

<style>
  .forge-property-inspector {
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-sm);
  }

  .forge-property-inspector__header {
    display: flex;
    align-items: center;
    gap: var(--forge-space-sm);
    margin-bottom: var(--forge-space-md);
  }

  .forge-property-inspector__kind {
    padding: 1px 6px;
    border-radius: var(--forge-radius-sm);
    background-color: var(--forge-bg-tertiary);
    color: var(--forge-text-muted);
    font-size: var(--forge-font-size-xs);
    text-transform: uppercase;
  }

  .forge-property-inspector__name {
    font-weight: 600;
    color: var(--forge-text-primary);
    font-size: var(--forge-font-size-md);
  }

  .forge-property-inspector__source-link {
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

  .forge-property-inspector__source-link:hover {
    text-decoration: underline;
  }

  .forge-property-inspector__props {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: var(--forge-space-xs) var(--forge-space-md);
    margin: 0;
  }

  .forge-property-inspector__key {
    color: var(--forge-text-secondary);
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-xs);
  }

  .forge-property-inspector__value {
    color: var(--forge-text-primary);
    margin: 0;
    word-break: break-word;
  }
</style>
