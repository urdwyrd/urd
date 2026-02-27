<script lang="ts" generics="T">
  /**
   * VirtualTable — virtualised table rendering only visible rows.
   *
   * Fixed 28px row height, CSS transform positioning, 5-row overscan.
   * Sorting lives in the parent — this is a pure render component.
   *
   * Layout: a sticky header row (div-based) above a scrollable body.
   * The body contains a spacer div for total height and visible rows
   * positioned via translateY.
   */

  import { onMount, onDestroy } from 'svelte';
  import type { ColumnDefinition, SortState } from './types';
  import ColumnHeader from './ColumnHeader.svelte';

  interface Props {
    rows: T[];
    columns: ColumnDefinition<T>[];
    sort: SortState | null;
    selectedRowIndex: number;
    emptyMessage?: string;
    rowKey?: (row: T, index: number) => string;
    onRowClick?: (row: T, index: number) => void;
    onRowDoubleClick?: (row: T, index: number) => void;
    onRowContextMenu?: (row: T, index: number, event: MouseEvent) => void;
    onSortChange: (sort: SortState) => void;
  }

  let {
    rows,
    columns,
    sort,
    selectedRowIndex = -1,
    emptyMessage = 'No data',
    rowKey,
    onRowClick,
    onRowDoubleClick,
    onRowContextMenu,
    onSortChange,
  }: Props = $props();

  const ROW_HEIGHT = 28;
  const OVERSCAN = 5;

  let scrollBody: HTMLDivElement | undefined = $state();
  let scrollTop: number = $state(0);
  let containerHeight: number = $state(400);

  let totalHeight = $derived(rows.length * ROW_HEIGHT);

  let startIndex = $derived(Math.max(0, Math.floor(scrollTop / ROW_HEIGHT) - OVERSCAN));
  let endIndex = $derived(
    Math.min(rows.length, Math.ceil((scrollTop + containerHeight) / ROW_HEIGHT) + OVERSCAN)
  );
  let visibleRows = $derived(rows.slice(startIndex, endIndex));
  let offsetY = $derived(startIndex * ROW_HEIGHT);

  function handleScroll(): void {
    if (scrollBody) {
      scrollTop = scrollBody.scrollTop;
    }
  }

  // ResizeObserver for container height
  let resizeObserver: ResizeObserver | undefined;

  onMount(() => {
    if (scrollBody) {
      containerHeight = scrollBody.clientHeight;
      resizeObserver = new ResizeObserver((entries) => {
        for (const entry of entries) {
          containerHeight = entry.contentRect.height;
        }
      });
      resizeObserver.observe(scrollBody);
    }
  });

  onDestroy(() => {
    resizeObserver?.disconnect();
  });

  function getCellValue(row: T, col: ColumnDefinition<T>): string {
    if (col.render) return col.render(row);
    const val = (row as Record<string, unknown>)[col.key];
    return val == null ? '' : String(val);
  }

  function handleRowKeydown(e: KeyboardEvent, row: T, idx: number): void {
    if (e.key === 'Enter') {
      onRowDoubleClick?.(row, startIndex + idx);
    }
  }
</script>

<div class="forge-virtual-table">
  {#if rows.length === 0}
    <div class="forge-virtual-table__empty">
      {emptyMessage}
    </div>
  {:else}
    <!-- Sticky header row -->
    <div class="forge-virtual-table__header" role="row">
      {#each columns as col}
        <ColumnHeader
          label={col.label}
          columnKey={col.key}
          sortable={col.sortable !== false}
          {sort}
          width={col.width}
          align={col.align}
          {onSortChange}
        />
      {/each}
    </div>

    <!-- Scrollable body -->
    <div
      class="forge-virtual-table__body"
      bind:this={scrollBody}
      onscroll={handleScroll}
      role="rowgroup"
    >
      <!-- Spacer to create correct scroll height -->
      <div class="forge-virtual-table__spacer" style="height: {totalHeight}px;">
        <!-- Visible rows positioned via translateY -->
        <div
          class="forge-virtual-table__rows"
          style="transform: translateY({offsetY}px)"
        >
          {#each visibleRows as row, idx (rowKey ? rowKey(row, startIndex + idx) : startIndex + idx)}
            <div
              class="forge-virtual-table__row"
              class:forge-virtual-table__row--selected={startIndex + idx === selectedRowIndex}
              class:forge-virtual-table__row--alt={(startIndex + idx) % 2 === 1}
              role="row"
              tabindex="0"
              onclick={() => onRowClick?.(row, startIndex + idx)}
              ondblclick={() => onRowDoubleClick?.(row, startIndex + idx)}
              oncontextmenu={(e) => { e.preventDefault(); onRowContextMenu?.(row, startIndex + idx, e); }}
              onkeydown={(e) => handleRowKeydown(e, row, idx)}
            >
              {#each columns as col}
                <span
                  class="forge-virtual-table__cell"
                  class:forge-virtual-table__cell--centre={col.align === 'centre'}
                  class:forge-virtual-table__cell--right={col.align === 'right'}
                  style:width={col.width}
                  style:min-width={col.width}
                  style:flex-shrink="0"
                >
                  {getCellValue(row, col)}
                </span>
              {/each}
            </div>
          {/each}
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .forge-virtual-table {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    overflow: hidden;
  }

  .forge-virtual-table__empty {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    color: var(--forge-text-muted);
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-md);
  }

  .forge-virtual-table__header {
    display: flex;
    flex-shrink: 0;
    border-bottom: 1px solid var(--forge-border-zone);
  }

  .forge-virtual-table__body {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    overflow-x: auto;
  }

  .forge-virtual-table__spacer {
    position: relative;
  }

  .forge-virtual-table__rows {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
  }

  .forge-virtual-table__row {
    display: flex;
    align-items: center;
    height: 28px;
    cursor: pointer;
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-sm);
    color: var(--forge-text-primary);
  }

  .forge-virtual-table__row:hover {
    background-color: var(--forge-table-row-hover);
  }

  .forge-virtual-table__row--alt {
    background-color: var(--forge-table-row-alt);
  }

  .forge-virtual-table__row--selected {
    background-color: var(--forge-accent-selection);
  }

  .forge-virtual-table__row:focus-visible {
    outline: 1px solid var(--forge-accent-primary);
    outline-offset: -1px;
  }

  .forge-virtual-table__cell {
    padding: 0 var(--forge-space-md);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .forge-virtual-table__cell--centre {
    text-align: center;
  }

  .forge-virtual-table__cell--right {
    text-align: right;
  }
</style>
