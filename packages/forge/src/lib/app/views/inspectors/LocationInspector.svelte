<script lang="ts">
  /**
   * LocationInspector — shows details of the selected location.
   *
   * Subscribes to selection.primary. When a location is selected, shows its
   * description, exits, contained entities, incoming exits, and source link.
   */

  import { onMount, onDestroy } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { selectionContext, type SelectionState } from '$lib/framework/selection/SelectionContext';
  import { projectionRegistry } from '$lib/app/projections/ProjectionRegistry';
  import { navigationBroker } from '$lib/framework/navigation/NavigationBroker';
  import type { UrdWorld, UrdExit, FactSet } from '$lib/app/compiler/types';
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

  let description: string = $state('');
  let exits: UrdExit[] = $state([]);
  let contains: string[] = $state([]);
  let incomingExits: { from: string; direction: string }[] = $state([]);

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
    if (state.items.length > 0 && state.items[0].kind === 'location') {
      const item = state.items[0];
      selectedId = item.id;
      selectedLabel = item.label ?? item.id;
      selectedData = (item.data as Record<string, unknown>) ?? null;
      refreshData();
    } else {
      selectedId = null;
      selectedLabel = null;
      selectedData = null;
      description = '';
      exits = [];
      contains = [];
      incomingExits = [];
    }
  }

  function refreshData(): void {
    if (!selectedId) {
      description = '';
      exits = [];
      contains = [];
      incomingExits = [];
      return;
    }

    const urdJson = projectionRegistry.get<UrdWorld>('urd.projection.urdJson');
    const location = urdJson?.locations.find((l) => l.id === selectedId);

    description = location?.description ?? '';
    exits = location?.exits ?? [];
    contains = location?.contains ?? [];

    const factSet = projectionRegistry.get<FactSet>('urd.projection.factSet');
    if (factSet) {
      incomingExits = factSet.exits
        .filter((e) => e.to_location === selectedId)
        .map((e) => ({ from: e.from_location, direction: e.exit_name }));
    } else {
      incomingExits = [];
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
  title="Location Inspector"
  emptyMessage="Select a location to inspect"
  hasContent={selectedId !== null}
>
  {#if selectedId}
    <div class="forge-location-inspector">
      <div class="forge-location-inspector__header">
        <span class="forge-location-inspector__kind">location</span>
        <span class="forge-location-inspector__name">{selectedLabel}</span>
      </div>

      {#if selectedData?.file}
        <button
          class="forge-location-inspector__source-link"
          onclick={goToSource}
        >
          {selectedData.file}:{selectedData.line ?? '?'} →
        </button>
      {/if}

      {#if description}
        <InspectorSection title="Description">
          <p class="forge-location-inspector__value">{description}</p>
        </InspectorSection>
      {/if}

      {#if exits.length > 0}
        <InspectorSection title="Exits ({exits.length})">
          <dl class="forge-location-inspector__props">
            {#each exits as exit}
              <dt class="forge-location-inspector__key">{exit.direction}</dt>
              <dd class="forge-location-inspector__value">{exit.target}</dd>
            {/each}
          </dl>
        </InspectorSection>
      {/if}

      {#if contains.length > 0}
        <InspectorSection title="Contains ({contains.length})">
          <ul class="forge-location-inspector__list">
            {#each contains as entityId}
              <li class="forge-location-inspector__list-item">{entityId}</li>
            {/each}
          </ul>
        </InspectorSection>
      {/if}

      {#if incomingExits.length > 0}
        <InspectorSection title="Incoming Exits ({incomingExits.length})">
          <dl class="forge-location-inspector__props">
            {#each incomingExits as incoming}
              <dt class="forge-location-inspector__key">{incoming.direction}</dt>
              <dd class="forge-location-inspector__value">{incoming.from}</dd>
            {/each}
          </dl>
        </InspectorSection>
      {/if}
    </div>
  {/if}
</InspectorPanel>

<style>
  .forge-location-inspector {
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-sm);
  }

  .forge-location-inspector__header {
    display: flex;
    align-items: center;
    gap: var(--forge-space-sm);
    margin-bottom: var(--forge-space-md);
  }

  .forge-location-inspector__kind {
    padding: 1px 6px;
    border-radius: var(--forge-radius-sm);
    background-color: var(--forge-bg-tertiary);
    color: var(--forge-text-muted);
    font-size: var(--forge-font-size-xs);
    text-transform: uppercase;
  }

  .forge-location-inspector__name {
    font-weight: 600;
    color: var(--forge-text-primary);
    font-size: var(--forge-font-size-md);
  }

  .forge-location-inspector__source-link {
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

  .forge-location-inspector__source-link:hover {
    text-decoration: underline;
  }

  .forge-location-inspector__props {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: var(--forge-space-xs) var(--forge-space-md);
    margin: 0;
  }

  .forge-location-inspector__key {
    color: var(--forge-text-secondary);
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-xs);
  }

  .forge-location-inspector__value {
    color: var(--forge-text-primary);
    margin: 0;
    word-break: break-word;
  }

  .forge-location-inspector__list {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .forge-location-inspector__list-item {
    padding: var(--forge-space-xs) 0;
    color: var(--forge-text-primary);
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-xs);
  }
</style>
