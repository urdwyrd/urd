<script lang="ts">
  /**
   * ReadSpreadsheet — table view of all property reads in the world.
   */

  import { onMount, onDestroy, untrack } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { projectionRegistry } from '$lib/app/projections/ProjectionRegistry';
  import { selectionContext } from '$lib/framework/selection/SelectionContext';
  import { navigationBroker } from '$lib/framework/navigation/NavigationBroker';
  import type { ReadRow } from '$lib/app/projections/read-table';
  import type { ColumnDefinition, SortState, VirtualTableState } from './_shared/types';
  import VirtualTable from './_shared/VirtualTable.svelte';
  import FilterBar from './_shared/FilterBar.svelte';

  interface Props {
    zoneId: string;
    zoneTypeId: string;
    state: VirtualTableState | null;
    onStateChange: (newState: unknown) => void;
  }

  let { zoneId, zoneTypeId, state: zoneState, onStateChange }: Props = $props();

  let rawRows: ReadRow[] = $state([]);
  let sort: SortState | null = $state(untrack(() => zoneState?.sort ?? null));
  let filterText: string = $state(untrack(() => zoneState?.filterText ?? ''));
  let selectedIndex: number = $state(-1);

  const columns: ColumnDefinition<ReadRow>[] = [
    { key: 'siteId', label: 'Site', width: '130px' },
    { key: 'siteKind', label: 'Kind', width: '80px' },
    { key: 'entityType', label: 'Entity Type', width: '120px' },
    { key: 'property', label: 'Property', width: '120px' },
    { key: 'operator', label: 'Op', width: '50px' },
    { key: 'valueLiteral', label: 'Value', width: '120px' },
    { key: 'file', label: 'File', width: '160px' },
    { key: 'line', label: 'Line', width: '60px', align: 'right' },
  ];

  let filteredRows = $derived.by(() => {
    let rows = rawRows;
    if (filterText) {
      const lower = filterText.toLowerCase();
      rows = rows.filter((r) =>
        r.entityType.toLowerCase().includes(lower) ||
        r.property.toLowerCase().includes(lower) ||
        r.siteId.toLowerCase().includes(lower)
      );
    }
    if (sort) {
      const { column, direction } = sort;
      const dir = direction === 'asc' ? 1 : -1;
      rows = [...rows].sort((a, b) => {
        const va = (a as Record<string, unknown>)[column];
        const vb = (b as Record<string, unknown>)[column];
        if (typeof va === 'number' && typeof vb === 'number') return (va - vb) * dir;
        return String(va ?? '').localeCompare(String(vb ?? '')) * dir;
      });
    }
    return rows;
  });

  function refreshData(): void {
    rawRows = projectionRegistry.get<ReadRow[]>('urd.projection.readTable') ?? [];
  }

  const unsubscribers: (() => void)[] = [];

  onMount(() => {
    refreshData();
    unsubscribers.push(bus.subscribe('compiler.completed', refreshData));
  });

  onDestroy(() => {
    for (const unsub of unsubscribers) unsub();
    persistState();
  });

  function persistState(): void {
    onStateChange({ sort, filterText } satisfies VirtualTableState);
  }

  function handleSortChange(newSort: SortState): void {
    sort = newSort;
    persistState();
  }

  function handleFilterChange(text: string): void {
    filterText = text;
  }

  function handleRowClick(row: ReadRow, index: number): void {
    selectedIndex = index;
    selectionContext.select([{ kind: 'property', id: `${row.entityType}.${row.property}`, label: `${row.entityType}.${row.property}`, data: { file: row.file, line: row.line } }], zoneId);
  }

  function handleRowDoubleClick(row: ReadRow): void {
    if (row.file && row.line) {
      navigationBroker.navigate({
        targetViewId: 'urd.codeEditor',
        params: { path: row.file, line: row.line },
      });
    }
  }
</script>

<div class="forge-spreadsheet-zone">
  <FilterBar {filterText} onFilterChange={handleFilterChange} placeholder="Filter reads\u2026" />
  <div class="forge-spreadsheet-zone__table">
    <VirtualTable
      rows={filteredRows}
      {columns}
      {sort}
      selectedRowIndex={selectedIndex}
      emptyMessage="No property reads"
      rowKey={(row) => row.id}
      onRowClick={handleRowClick}
      onRowDoubleClick={handleRowDoubleClick}
      onSortChange={handleSortChange}
    />
  </div>
</div>

<style>
  .forge-spreadsheet-zone {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    overflow: hidden;
  }

  .forge-spreadsheet-zone__table {
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }
</style>
