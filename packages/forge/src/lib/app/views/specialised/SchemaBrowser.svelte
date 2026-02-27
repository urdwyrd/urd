<script lang="ts">
  /**
   * SchemaBrowser — tree browser for urdJson.types.
   *
   * Reads urd.projection.urdJson. Shows collapsible type entries, each
   * with properties (name, type, default). Filter input at top. Click
   * property to select it via selectionContext.
   */

  import { onMount, onDestroy } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { projectionRegistry } from '$lib/app/projections/ProjectionRegistry';
  import { selectionContext } from '$lib/framework/selection/SelectionContext';
  import type { UrdWorld, UrdTypeDef, UrdPropertyDef } from '$lib/app/compiler/types';

  interface Props {
    zoneId: string;
    zoneTypeId: string;
    state: null;
    onStateChange: (newState: unknown) => void;
  }

  let { zoneId, zoneTypeId, state: zoneState, onStateChange }: Props = $props();

  interface TypeEntry {
    name: string;
    traits: string[];
    properties: PropertyEntry[];
  }

  interface PropertyEntry {
    name: string;
    type: string;
    defaultValue: string;
    typeName: string;
  }

  let types: TypeEntry[] = $state([]);
  let filter = $state('');
  let expandedTypes = $state(new Set<string>());

  const unsubscribers: (() => void)[] = [];

  onMount(() => {
    refreshData();
    unsubscribers.push(bus.subscribe('compiler.completed', refreshData));
  });

  onDestroy(() => {
    for (const unsub of unsubscribers) unsub();
  });

  function refreshData(): void {
    const urdJson = projectionRegistry.get<UrdWorld>('urd.projection.urdJson');
    if (!urdJson?.types) {
      types = [];
      return;
    }

    const typeEntries: TypeEntry[] = [];
    for (const [typeName, typeDef] of Object.entries(urdJson.types)) {
      const properties: PropertyEntry[] = [];
      if (typeDef.properties) {
        for (const [propName, propDef] of Object.entries(typeDef.properties)) {
          properties.push({
            name: propName,
            type: propDef.type ?? 'unknown',
            defaultValue: propDef.default !== undefined ? String(propDef.default) : '—',
            typeName,
          });
        }
      }
      properties.sort((a, b) => a.name.localeCompare(b.name));

      typeEntries.push({
        name: typeName,
        traits: typeDef.traits ?? [],
        properties,
      });
    }
    typeEntries.sort((a, b) => a.name.localeCompare(b.name));
    types = typeEntries;
  }

  let filteredTypes = $derived.by((): TypeEntry[] => {
    if (!filter.trim()) return types;
    const q = filter.toLowerCase();
    return types.filter((t) =>
      t.name.toLowerCase().includes(q) ||
      t.properties.some((p) => p.name.toLowerCase().includes(q)),
    );
  });

  function toggleType(name: string): void {
    const next = new Set(expandedTypes);
    if (next.has(name)) {
      next.delete(name);
    } else {
      next.add(name);
    }
    expandedTypes = next;
  }

  function handlePropertyClick(prop: PropertyEntry): void {
    selectionContext.select([
      {
        kind: 'property',
        id: `${prop.typeName}.${prop.name}`,
        label: `${prop.typeName}.${prop.name}`,
        data: { type: prop.type, default: prop.defaultValue },
      },
    ]);
  }
</script>

<div class="forge-schema-browser">
  <div class="forge-schema-browser__toolbar">
    <span class="forge-schema-browser__title">Schema Browser</span>
    <div class="forge-schema-browser__spacer"></div>
    <span class="forge-schema-browser__count">
      {filteredTypes.length} type{filteredTypes.length !== 1 ? 's' : ''}
    </span>
  </div>

  <div class="forge-schema-browser__filter">
    <input
      class="forge-schema-browser__filter-input"
      type="text"
      placeholder="Filter types and properties..."
      bind:value={filter}
    />
  </div>

  {#if types.length === 0}
    <div class="forge-schema-browser__empty">
      <p>No type definitions available</p>
      <p class="forge-schema-browser__hint">Compile a project with type definitions to browse the schema</p>
    </div>
  {:else if filteredTypes.length === 0}
    <div class="forge-schema-browser__empty">
      <p>No types match "{filter}"</p>
    </div>
  {:else}
    <div class="forge-schema-browser__list">
      {#each filteredTypes as typeEntry}
        <div class="forge-schema-browser__type">
          <button
            class="forge-schema-browser__type-header"
            onclick={() => toggleType(typeEntry.name)}
          >
            <span class="forge-schema-browser__type-arrow">
              {expandedTypes.has(typeEntry.name) ? '▾' : '▸'}
            </span>
            <span class="forge-schema-browser__type-name">{typeEntry.name}</span>
            {#if typeEntry.traits.length > 0}
              <span class="forge-schema-browser__type-traits">
                {typeEntry.traits.join(', ')}
              </span>
            {/if}
            <span class="forge-schema-browser__type-prop-count">
              {typeEntry.properties.length}
            </span>
          </button>

          {#if expandedTypes.has(typeEntry.name)}
            <div class="forge-schema-browser__properties">
              {#if typeEntry.properties.length === 0}
                <div class="forge-schema-browser__no-props">No properties</div>
              {:else}
                {#each typeEntry.properties as prop}
                  <button
                    class="forge-schema-browser__property"
                    onclick={() => handlePropertyClick(prop)}
                  >
                    <span class="forge-schema-browser__prop-name">{prop.name}</span>
                    <span class="forge-schema-browser__prop-type">{prop.type}</span>
                    {#if prop.defaultValue !== '—'}
                      <span class="forge-schema-browser__prop-default">= {prop.defaultValue}</span>
                    {/if}
                  </button>
                {/each}
              {/if}
            </div>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .forge-schema-browser {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    overflow: hidden;
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-sm);
  }

  .forge-schema-browser__toolbar {
    display: flex;
    align-items: center;
    gap: var(--forge-space-sm);
    height: 28px;
    padding: 0 var(--forge-space-md);
    background-color: var(--forge-bg-secondary);
    border-bottom: 1px solid var(--forge-border-zone);
    flex-shrink: 0;
  }

  .forge-schema-browser__title {
    font-size: var(--forge-font-size-xs);
    font-weight: 600;
    color: var(--forge-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .forge-schema-browser__spacer {
    flex: 1;
  }

  .forge-schema-browser__count {
    font-size: var(--forge-font-size-xs);
    color: var(--forge-text-muted);
  }

  .forge-schema-browser__filter {
    padding: var(--forge-space-xs) var(--forge-space-md);
    border-bottom: 1px solid var(--forge-border-zone);
    flex-shrink: 0;
  }

  .forge-schema-browser__filter-input {
    width: 100%;
    padding: var(--forge-space-xs) var(--forge-space-sm);
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-sm);
    background: var(--forge-bg-tertiary);
    color: var(--forge-text-primary);
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-xs);
  }

  .forge-schema-browser__filter-input::placeholder {
    color: var(--forge-text-muted);
  }

  .forge-schema-browser__empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    flex: 1;
    gap: var(--forge-space-sm);
    color: var(--forge-text-muted);
  }

  .forge-schema-browser__hint {
    font-size: var(--forge-font-size-xs);
    opacity: 0.6;
  }

  .forge-schema-browser__list {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
  }

  .forge-schema-browser__type {
    border-bottom: 1px solid var(--forge-border-subtle, rgba(255, 255, 255, 0.04));
  }

  .forge-schema-browser__type-header {
    display: flex;
    align-items: center;
    gap: var(--forge-space-sm);
    width: 100%;
    padding: var(--forge-space-sm) var(--forge-space-md);
    background: none;
    border: none;
    cursor: pointer;
    text-align: left;
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-sm);
    color: var(--forge-text-primary);
  }

  .forge-schema-browser__type-header:hover {
    background: var(--forge-bg-hover);
  }

  .forge-schema-browser__type-arrow {
    width: 12px;
    flex-shrink: 0;
    font-size: 10px;
    color: var(--forge-text-muted);
  }

  .forge-schema-browser__type-name {
    font-weight: 600;
    color: var(--forge-text-primary);
  }

  .forge-schema-browser__type-traits {
    font-size: var(--forge-font-size-xs);
    color: var(--forge-text-muted);
    font-style: italic;
  }

  .forge-schema-browser__type-prop-count {
    margin-left: auto;
    font-size: 10px;
    font-family: var(--forge-font-family-mono);
    color: var(--forge-text-muted);
    background: var(--forge-bg-tertiary);
    padding: 0 4px;
    border-radius: var(--forge-radius-sm);
  }

  .forge-schema-browser__properties {
    padding-left: calc(var(--forge-space-md) + 12px + var(--forge-space-sm));
    padding-bottom: var(--forge-space-xs);
  }

  .forge-schema-browser__no-props {
    font-size: var(--forge-font-size-xs);
    color: var(--forge-text-muted);
    font-style: italic;
    padding: var(--forge-space-xs) 0;
  }

  .forge-schema-browser__property {
    display: flex;
    align-items: center;
    gap: var(--forge-space-sm);
    width: 100%;
    padding: var(--forge-space-xs) var(--forge-space-sm);
    background: none;
    border: none;
    border-radius: var(--forge-radius-sm);
    cursor: pointer;
    text-align: left;
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-xs);
    color: var(--forge-text-primary);
  }

  .forge-schema-browser__property:hover {
    background: var(--forge-bg-hover);
  }

  .forge-schema-browser__prop-name {
    font-family: var(--forge-font-family-mono);
    color: var(--forge-text-primary);
  }

  .forge-schema-browser__prop-type {
    color: var(--forge-accent-primary, #5b9bd5);
    font-size: 10px;
  }

  .forge-schema-browser__prop-default {
    color: var(--forge-text-muted);
    font-size: 10px;
    font-family: var(--forge-font-family-mono);
  }
</style>
