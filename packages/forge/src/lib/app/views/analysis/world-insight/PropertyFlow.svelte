<script lang="ts">
  /**
   * PropertyFlow — property dependency list with filter modes (Section 3).
   */

  import type { PropertyDependencyIndex, FactSet } from '$lib/app/compiler/types';
  import PropertyFlowRow from './PropertyFlowRow.svelte';

  type FilterMode = 'all' | 'read_only' | 'write_only' | 'orphaned';

  interface Props {
    propertyIndex: PropertyDependencyIndex | null;
    factSet: FactSet | null;
    expandedRows: Record<string, boolean>;
    onToggleRow: (key: string) => void;
    onNavigate: (path: string, line: number) => void;
    filterText: string;
    filterMode: FilterMode;
    onFilterModeChange: (mode: FilterMode) => void;
  }

  let { propertyIndex, factSet, expandedRows, onToggleRow, onNavigate, filterText, filterMode, onFilterModeChange }: Props = $props();

  let filteredProperties = $derived.by(() => {
    if (!propertyIndex) return [];

    let props = propertyIndex.properties;
    const q = filterText.toLowerCase();

    // Apply text filter
    if (q) {
      props = props.filter((p) => {
        const key = `${p.entity_type}.${p.property}`;
        return key.toLowerCase().includes(q);
      });
    }

    // Apply mode filter
    switch (filterMode) {
      case 'read_only':
        props = props.filter((p) => p.read_count > 0 && p.write_count === 0);
        break;
      case 'write_only':
        props = props.filter((p) => p.write_count > 0 && p.read_count === 0);
        break;
      case 'orphaned':
        props = props.filter((p) => p.orphaned !== null);
        break;
    }

    return props;
  });

  const filterButtons: { mode: FilterMode; label: string }[] = [
    { mode: 'all', label: 'All' },
    { mode: 'read_only', label: 'Read-only' },
    { mode: 'write_only', label: 'Write-only' },
    { mode: 'orphaned', label: 'Orphaned' },
  ];
</script>

<div class="forge-property-flow">
  {#if propertyIndex}
    <div class="forge-property-flow__summary">
      <span>{propertyIndex.summary.total_properties} properties</span>
      <span class="forge-property-flow__sep">\u00B7</span>
      <span>{propertyIndex.summary.total_reads} reads</span>
      <span class="forge-property-flow__sep">\u00B7</span>
      <span>{propertyIndex.summary.total_writes} writes</span>
    </div>
    <div class="forge-property-flow__filters">
      {#each filterButtons as btn}
        <button
          class="forge-property-flow__filter-btn"
          class:forge-property-flow__filter-btn--active={filterMode === btn.mode}
          onclick={() => onFilterModeChange(btn.mode)}
        >
          {btn.label}
        </button>
      {/each}
    </div>
  {/if}

  {#if filteredProperties.length === 0}
    <div class="forge-property-flow__empty">No properties</div>
  {:else if factSet}
    {#each filteredProperties as property}
      <PropertyFlowRow
        {property}
        {factSet}
        expanded={expandedRows[`prop:${property.entity_type}.${property.property}`] ?? false}
        onToggle={onToggleRow}
        {onNavigate}
      />
    {/each}
  {/if}
</div>

<style>
  .forge-property-flow {
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-xs);
  }

  .forge-property-flow__summary {
    display: flex;
    align-items: center;
    gap: var(--forge-space-xs);
    padding: var(--forge-space-xs) var(--forge-space-sm);
    color: var(--forge-text-muted);
    font-size: 10px;
    border-bottom: 1px solid color-mix(in srgb, var(--forge-border-zone) 50%, transparent);
  }

  .forge-property-flow__sep {
    color: var(--forge-text-muted);
  }

  .forge-property-flow__filters {
    display: flex;
    gap: 2px;
    padding: var(--forge-space-xs) var(--forge-space-sm);
    border-bottom: 1px solid color-mix(in srgb, var(--forge-border-zone) 50%, transparent);
  }

  .forge-property-flow__filter-btn {
    font-family: var(--forge-font-family-ui);
    font-size: 10px;
    padding: 1px var(--forge-space-xs);
    border-radius: var(--forge-radius-sm);
    background: none;
    border: 1px solid transparent;
    color: var(--forge-text-muted);
    cursor: pointer;
  }

  .forge-property-flow__filter-btn:hover {
    color: var(--forge-text-secondary);
    border-color: var(--forge-border-zone);
  }

  .forge-property-flow__filter-btn--active {
    color: var(--forge-text-primary);
    border-color: var(--forge-text-muted);
    background-color: color-mix(in srgb, var(--forge-text-muted) 10%, transparent);
  }

  .forge-property-flow__empty {
    padding: var(--forge-space-md);
    color: var(--forge-text-muted);
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-sm);
    text-align: center;
  }
</style>
