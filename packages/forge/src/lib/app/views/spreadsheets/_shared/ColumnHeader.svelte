<script lang="ts">
  /**
   * ColumnHeader — clickable header cell with sort direction indicator.
   */

  import type { SortState } from './types';

  interface Props {
    label: string;
    columnKey: string;
    sortable?: boolean;
    sort: SortState | null;
    width?: string;
    align?: 'left' | 'centre' | 'right';
    onSortChange: (sort: SortState) => void;
  }

  let { label, columnKey, sortable = true, sort, width, align = 'left', onSortChange }: Props = $props();

  let isActive = $derived(sort?.column === columnKey);
  let indicator = $derived(isActive ? (sort!.direction === 'asc' ? ' \u25B2' : ' \u25BC') : '');

  function handleClick(): void {
    if (!sortable) return;
    if (isActive) {
      onSortChange({ column: columnKey, direction: sort!.direction === 'asc' ? 'desc' : 'asc' });
    } else {
      onSortChange({ column: columnKey, direction: 'asc' });
    }
  }

  function handleKeydown(e: KeyboardEvent): void {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      handleClick();
    }
  }
</script>

<div
  class="forge-column-header"
  class:forge-column-header--sortable={sortable}
  class:forge-column-header--active={isActive}
  class:forge-column-header--centre={align === 'centre'}
  class:forge-column-header--right={align === 'right'}
  style:width={width}
  style:min-width={width}
  style:flex-shrink="0"
  onclick={handleClick}
  onkeydown={handleKeydown}
  role="columnheader"
  aria-sort={isActive ? (sort!.direction === 'asc' ? 'ascending' : 'descending') : undefined}
  tabindex={sortable ? 0 : undefined}
>
  {label}{indicator}
</div>

<style>
  .forge-column-header {
    height: 28px;
    padding: 0 var(--forge-space-md);
    background-color: var(--forge-table-header-bg);
    color: var(--forge-text-secondary);
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-xs);
    font-weight: 600;
    text-align: left;
    white-space: nowrap;
    user-select: none;
    display: flex;
    align-items: center;
  }

  .forge-column-header--sortable {
    cursor: pointer;
  }

  .forge-column-header--sortable:hover {
    color: var(--forge-text-primary);
  }

  .forge-column-header--active {
    color: var(--forge-accent-primary);
  }

  .forge-column-header--centre {
    text-align: center;
    justify-content: center;
  }

  .forge-column-header--right {
    text-align: right;
    justify-content: flex-end;
  }
</style>
