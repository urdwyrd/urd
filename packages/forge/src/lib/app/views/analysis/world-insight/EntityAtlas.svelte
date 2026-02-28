<script lang="ts">
  /**
   * EntityAtlas — type-grouped entity tree (Section 2).
   * L0: type group rows with count and trait badges.
   * L1: individual EntityRow components.
   */

  import type {
    UrdWorld,
    UrdEntity,
    FactSet,
    PropertyDependencyIndex,
    SymbolTableEntry,
  } from '$lib/app/compiler/types';
  import EntityRow from './EntityRow.svelte';

  interface Props {
    world: UrdWorld;
    symbolMap: Map<string, SymbolTableEntry>;
    propertyIndex: PropertyDependencyIndex | null;
    factSet: FactSet | null;
    entityLocationMap: Map<string, string>;
    expandedRows: Record<string, boolean>;
    onToggleRow: (key: string) => void;
    onNavigate: (path: string, line: number) => void;
    filterText: string;
    entityTypeFilter: string | null;
    onEntityTypeFilterChange: (type: string | null) => void;
    highlightedEntityId?: string | null;
  }

  let {
    world, symbolMap, propertyIndex, factSet, entityLocationMap,
    expandedRows, onToggleRow, onNavigate, filterText,
    entityTypeFilter, onEntityTypeFilterChange, highlightedEntityId,
  }: Props = $props();

  // Group entities by type
  let typeGroups = $derived.by(() => {
    const groups = new Map<string, UrdEntity[]>();
    const q = filterText.toLowerCase();

    for (const entity of world.entities) {
      const type = entity.type ?? '(untyped)';

      // Apply type filter
      if (entityTypeFilter && type !== entityTypeFilter) continue;

      // Apply text filter
      if (q) {
        const matches =
          entity.id.toLowerCase().includes(q) ||
          entity.name.toLowerCase().includes(q) ||
          type.toLowerCase().includes(q) ||
          Object.entries(entity.properties).some(
            ([k, v]) => k.toLowerCase().includes(q) || String(v).toLowerCase().includes(q),
          );
        if (!matches) continue;
      }

      const list = groups.get(type) ?? [];
      list.push(entity);
      groups.set(type, list);
    }

    return [...groups.entries()].sort((a, b) => a[0].localeCompare(b[0]));
  });

  // All unique type names for the filter dropdown
  let allTypes = $derived.by(() => {
    const types = new Set<string>();
    for (const entity of world.entities) {
      types.add(entity.type ?? '(untyped)');
    }
    return [...types].sort();
  });

  function getTraits(typeName: string): string[] {
    return world.types?.[typeName]?.traits ?? [];
  }

  function getPropertyDeps(typeName: string) {
    if (!propertyIndex) return [];
    return propertyIndex.properties.filter((p) => p.entity_type === typeName);
  }

  function getLocationName(entityId: string): string | null {
    const locId = entityLocationMap.get(entityId);
    if (!locId) return null;
    const loc = world.locations.find((l) => l.id === locId);
    return loc?.name ?? locId;
  }

  function toggleTypeGroup(typeName: string): void {
    onToggleRow(`type:${typeName}`);
  }

  function handleFilterChange(e: Event): void {
    const value = (e.target as HTMLSelectElement).value;
    onEntityTypeFilterChange(value === '__all__' ? null : value);
  }
</script>

<div class="forge-entity-atlas">
  {#if allTypes.length > 1}
    <div class="forge-entity-atlas__filter-bar">
      <select
        class="forge-entity-atlas__type-select"
        value={entityTypeFilter ?? '__all__'}
        onchange={handleFilterChange}
      >
        <option value="__all__">All types</option>
        {#each allTypes as typeName}
          <option value={typeName}>{typeName}</option>
        {/each}
      </select>
    </div>
  {/if}

  {#if typeGroups.length === 0}
    <div class="forge-entity-atlas__empty">No entities</div>
  {:else}
    {#each typeGroups as [typeName, entities]}
      {@const isExpanded = expandedRows[`type:${typeName}`] ?? false}
      {@const traits = getTraits(typeName)}
      {@const deps = getPropertyDeps(typeName)}
      <div class="forge-entity-atlas__type-group">
        <button class="forge-entity-atlas__type-header" onclick={() => toggleTypeGroup(typeName)}>
          <span class="forge-entity-atlas__toggle">{isExpanded ? '\u25BE' : '\u25B8'}</span>
          <span class="forge-entity-atlas__type-name">{typeName}</span>
          <span class="forge-entity-atlas__type-count">({entities.length})</span>
          {#each traits as trait}
            <span class="forge-entity-atlas__trait-badge">{trait}</span>
          {/each}
        </button>

        {#if isExpanded}
          <div class="forge-entity-atlas__entity-list">
            {#each entities as entity}
              <div
                class="forge-entity-atlas__entity-wrapper"
                class:forge-entity-atlas__entity-wrapper--highlighted={highlightedEntityId === entity.id}
              >
                <EntityRow
                  {entity}
                  {world}
                  symbolEntry={symbolMap.get(entity.id) ?? symbolMap.get(entity.name)}
                  locationName={getLocationName(entity.id)}
                  propertyDeps={deps}
                  {factSet}
                  expanded={expandedRows[`entity:${entity.id}`] ?? false}
                  onToggle={onToggleRow}
                  {onNavigate}
                />
              </div>
            {/each}
          </div>
        {/if}
      </div>
    {/each}
  {/if}
</div>

<style>
  .forge-entity-atlas {
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-xs);
  }

  .forge-entity-atlas__filter-bar {
    padding: var(--forge-space-xs) var(--forge-space-sm);
    border-bottom: 1px solid color-mix(in srgb, var(--forge-border-zone) 50%, transparent);
  }

  .forge-entity-atlas__type-select {
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-xs);
    background-color: var(--forge-bg-secondary);
    color: var(--forge-text-secondary);
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-sm);
    padding: 2px var(--forge-space-xs);
  }

  .forge-entity-atlas__empty {
    padding: var(--forge-space-md);
    color: var(--forge-text-muted);
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-sm);
    text-align: center;
  }

  .forge-entity-atlas__type-group {
    border-bottom: 1px solid color-mix(in srgb, var(--forge-border-zone) 50%, transparent);
  }

  .forge-entity-atlas__type-header {
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

  .forge-entity-atlas__type-header:hover {
    background-color: var(--forge-table-row-hover);
  }

  .forge-entity-atlas__toggle {
    flex-shrink: 0;
    width: 10px;
    font-size: 10px;
    color: var(--forge-text-muted);
  }

  .forge-entity-atlas__type-name {
    font-weight: 600;
    color: var(--forge-text-primary);
  }

  .forge-entity-atlas__type-count {
    color: var(--forge-text-muted);
  }

  .forge-entity-atlas__trait-badge {
    font-size: 10px;
    padding: 0 3px;
    border-radius: var(--forge-radius-sm);
    background-color: color-mix(in srgb, var(--forge-text-muted) 12%, transparent);
    color: var(--forge-text-muted);
    line-height: 14px;
  }

  .forge-entity-atlas__entity-list {
    padding-left: 16px;
  }

  .forge-entity-atlas__entity-wrapper--highlighted {
    border-left: 3px solid var(--forge-status-warning);
  }
</style>
