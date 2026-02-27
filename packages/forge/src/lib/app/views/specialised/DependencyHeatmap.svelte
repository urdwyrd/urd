<script lang="ts">
  /**
   * DependencyHeatmap — heatmap grid of property dependencies.
   *
   * Rows = entity types, columns = properties. Cell colour intensity
   * = read+write count from propertyDependencyIndex projection.
   * Simple CSS grid with hover tooltip showing counts.
   */

  import { onMount, onDestroy } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { projectionRegistry } from '$lib/app/projections/ProjectionRegistry';
  import type { PropertyDependencyIndex, PropertyDependencyEntry } from '$lib/app/compiler/types';

  interface Props {
    zoneId: string;
    zoneTypeId: string;
    state: null;
    onStateChange: (newState: unknown) => void;
  }

  let { zoneId, zoneTypeId, state: zoneState, onStateChange }: Props = $props();

  interface HeatmapCell {
    entityType: string;
    property: string;
    readCount: number;
    writeCount: number;
    total: number;
    intensity: number;
  }

  let entityTypes: string[] = $state([]);
  let properties: string[] = $state([]);
  let cells: Map<string, HeatmapCell> = $state(new Map());
  let maxTotal = $state(1);
  let hoveredCell: HeatmapCell | null = $state(null);
  let hoverX = $state(0);
  let hoverY = $state(0);

  const unsubscribers: (() => void)[] = [];

  onMount(() => {
    refreshData();
    unsubscribers.push(bus.subscribe('compiler.completed', refreshData));
  });

  onDestroy(() => {
    for (const unsub of unsubscribers) unsub();
  });

  function refreshData(): void {
    const depIndex = projectionRegistry.get<PropertyDependencyIndex>('urd.projection.propertyDependencyIndex');
    if (!depIndex || depIndex.properties.length === 0) {
      entityTypes = [];
      properties = [];
      cells = new Map();
      return;
    }

    const typeSet = new Set<string>();
    const propSet = new Set<string>();
    const newCells = new Map<string, HeatmapCell>();
    let localMax = 1;

    for (const entry of depIndex.properties) {
      typeSet.add(entry.entity_type);
      propSet.add(entry.property);

      const total = entry.read_count + entry.write_count;
      if (total > localMax) localMax = total;

      const key = `${entry.entity_type}::${entry.property}`;
      newCells.set(key, {
        entityType: entry.entity_type,
        property: entry.property,
        readCount: entry.read_count,
        writeCount: entry.write_count,
        total,
        intensity: 0, // Computed below
      });
    }

    // Compute intensity (0..1)
    for (const cell of newCells.values()) {
      cell.intensity = cell.total / localMax;
    }

    entityTypes = [...typeSet].sort();
    properties = [...propSet].sort();
    cells = newCells;
    maxTotal = localMax;
  }

  function getCell(entityType: string, property: string): HeatmapCell | undefined {
    return cells.get(`${entityType}::${property}`);
  }

  function cellBackground(intensity: number): string {
    if (intensity === 0) return 'transparent';
    // Map intensity to opacity of the accent colour
    const alpha = Math.max(0.08, intensity * 0.8);
    return `rgba(91, 155, 213, ${alpha})`;
  }

  function handleCellHover(cell: HeatmapCell | undefined, event: MouseEvent): void {
    if (!cell || cell.total === 0) {
      hoveredCell = null;
      return;
    }
    hoveredCell = cell;
    hoverX = event.clientX;
    hoverY = event.clientY;
  }

  function handleCellLeave(): void {
    hoveredCell = null;
  }
</script>

<div class="forge-dep-heatmap">
  <div class="forge-dep-heatmap__toolbar">
    <span class="forge-dep-heatmap__title">Dependency Heatmap</span>
    <div class="forge-dep-heatmap__spacer"></div>
    {#if entityTypes.length > 0}
      <span class="forge-dep-heatmap__count">
        {entityTypes.length} type{entityTypes.length !== 1 ? 's' : ''} ·
        {properties.length} propert{properties.length !== 1 ? 'ies' : 'y'}
      </span>
    {/if}
  </div>

  {#if entityTypes.length === 0}
    <div class="forge-dep-heatmap__empty">
      <p>No property dependency data available</p>
      <p class="forge-dep-heatmap__hint">Compile a project with typed properties to see the heatmap</p>
    </div>
  {:else}
    <div class="forge-dep-heatmap__content">
      <div
        class="forge-dep-heatmap__grid"
        style="grid-template-columns: auto repeat({properties.length}, 1fr)"
      >
        <!-- Header row: empty corner + property names -->
        <div class="forge-dep-heatmap__corner"></div>
        {#each properties as prop}
          <div class="forge-dep-heatmap__col-header" title={prop}>
            {prop.length > 10 ? prop.slice(0, 8) + '..' : prop}
          </div>
        {/each}

        <!-- Data rows -->
        {#each entityTypes as entityType}
          <div class="forge-dep-heatmap__row-header" title={entityType}>
            {entityType.length > 14 ? entityType.slice(0, 12) + '..' : entityType}
          </div>
          {#each properties as prop}
            {@const cell = getCell(entityType, prop)}
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div
              class="forge-dep-heatmap__cell"
              style="background-color: {cellBackground(cell?.intensity ?? 0)}"
              onmouseenter={(e) => handleCellHover(cell, e)}
              onmouseleave={handleCellLeave}
            >
              {#if cell && cell.total > 0}
                <span class="forge-dep-heatmap__cell-value">{cell.total}</span>
              {/if}
            </div>
          {/each}
        {/each}
      </div>
    </div>
  {/if}

  {#if hoveredCell}
    <div
      class="forge-dep-heatmap__tooltip"
      style="left: {hoverX + 12}px; top: {hoverY - 8}px"
    >
      <div class="forge-dep-heatmap__tooltip-title">
        {hoveredCell.entityType}.{hoveredCell.property}
      </div>
      <div class="forge-dep-heatmap__tooltip-detail">
        Reads: {hoveredCell.readCount} · Writes: {hoveredCell.writeCount}
      </div>
    </div>
  {/if}
</div>

<style>
  .forge-dep-heatmap {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    overflow: hidden;
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-sm);
    position: relative;
  }

  .forge-dep-heatmap__toolbar {
    display: flex;
    align-items: center;
    gap: var(--forge-space-sm);
    height: 28px;
    padding: 0 var(--forge-space-md);
    background-color: var(--forge-bg-secondary);
    border-bottom: 1px solid var(--forge-border-zone);
    flex-shrink: 0;
  }

  .forge-dep-heatmap__title {
    font-size: var(--forge-font-size-xs);
    font-weight: 600;
    color: var(--forge-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .forge-dep-heatmap__spacer {
    flex: 1;
  }

  .forge-dep-heatmap__count {
    font-size: var(--forge-font-size-xs);
    color: var(--forge-text-muted);
  }

  .forge-dep-heatmap__empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    flex: 1;
    gap: var(--forge-space-sm);
    color: var(--forge-text-muted);
  }

  .forge-dep-heatmap__hint {
    font-size: var(--forge-font-size-xs);
    opacity: 0.6;
  }

  .forge-dep-heatmap__content {
    flex: 1;
    min-height: 0;
    overflow: auto;
    padding: var(--forge-space-md);
  }

  .forge-dep-heatmap__grid {
    display: grid;
    gap: 1px;
  }

  .forge-dep-heatmap__corner {
    background: var(--forge-bg-secondary);
  }

  .forge-dep-heatmap__col-header {
    padding: var(--forge-space-xs);
    font-size: 10px;
    font-weight: 600;
    color: var(--forge-text-secondary);
    text-align: center;
    background: var(--forge-bg-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    writing-mode: vertical-lr;
    transform: rotate(180deg);
    min-height: 60px;
  }

  .forge-dep-heatmap__row-header {
    padding: var(--forge-space-xs) var(--forge-space-sm);
    font-size: var(--forge-font-size-xs);
    font-weight: 600;
    color: var(--forge-text-secondary);
    background: var(--forge-bg-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    display: flex;
    align-items: center;
  }

  .forge-dep-heatmap__cell {
    display: flex;
    align-items: center;
    justify-content: center;
    min-width: 32px;
    min-height: 28px;
    border: 1px solid var(--forge-border-subtle, rgba(255, 255, 255, 0.04));
    cursor: default;
  }

  .forge-dep-heatmap__cell:hover {
    outline: 1px solid var(--forge-accent-primary, #5b9bd5);
  }

  .forge-dep-heatmap__cell-value {
    font-size: 10px;
    font-family: var(--forge-font-family-mono);
    color: var(--forge-text-primary);
  }

  .forge-dep-heatmap__tooltip {
    position: fixed;
    z-index: 1000;
    padding: var(--forge-space-sm) var(--forge-space-md);
    background: var(--forge-bg-secondary);
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-sm);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
    pointer-events: none;
  }

  .forge-dep-heatmap__tooltip-title {
    font-size: var(--forge-font-size-xs);
    font-weight: 600;
    color: var(--forge-text-primary);
    font-family: var(--forge-font-family-mono);
    margin-bottom: 2px;
  }

  .forge-dep-heatmap__tooltip-detail {
    font-size: 10px;
    color: var(--forge-text-secondary);
  }
</style>
