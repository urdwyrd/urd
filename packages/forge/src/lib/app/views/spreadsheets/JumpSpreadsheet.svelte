<script lang="ts">
  /**
   * JumpSpreadsheet — table view of all jump edges in the world.
   */

  import { onMount, onDestroy, untrack } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { projectionRegistry } from '$lib/app/projections/ProjectionRegistry';
  import { selectionContext } from '$lib/framework/selection/SelectionContext';
  import { navigationBroker } from '$lib/framework/navigation/NavigationBroker';
  import type { JumpRow } from '$lib/app/projections/jump-table';
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

  let rawRows: JumpRow[] = $state([]);
  let sort: SortState | null = $state(untrack(() => zoneState?.sort ?? null));
  let filterText: string = $state(untrack(() => zoneState?.filterText ?? ''));
  let selectedIndex: number = $state(-1);

  const columns: ColumnDefinition<JumpRow>[] = [
    { key: 'fromSection', label: 'From Section', width: '180px' },
    { key: 'targetKind', label: 'Target Kind', width: '100px' },
    { key: 'targetId', label: 'Target ID', width: '180px' },
    { key: 'file', label: 'File', width: '180px' },
    { key: 'line', label: 'Line', width: '60px', align: 'right' },
  ];

  let filteredRows = $derived.by(() => {
    let rows = rawRows;
    if (filterText) {
      const lower = filterText.toLowerCase();
      rows = rows.filter((r) =>
        r.fromSection.toLowerCase().includes(lower) || r.targetId.toLowerCase().includes(lower)
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
    rawRows = projectionRegistry.get<JumpRow[]>('urd.projection.jumpTable') ?? [];
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

  function handleRowClick(row: JumpRow, index: number): void {
    selectedIndex = index;
    selectionContext.select([{ kind: 'section', id: row.fromSection, label: row.fromSection, data: { file: row.file, line: row.line } }], zoneId);
  }

  function handleRowDoubleClick(row: JumpRow): void {
    if (row.file && row.line) {
      navigationBroker.navigate({
        targetViewId: 'urd.codeEditor',
        params: { path: row.file, line: row.line },
      });
    }
  }
</script>

<div class="forge-spreadsheet-zone">
  <FilterBar {filterText} onFilterChange={handleFilterChange} placeholder="Filter jumps\u2026" />
  <div class="forge-spreadsheet-zone__table">
    <VirtualTable
      rows={filteredRows}
      {columns}
      {sort}
      selectedRowIndex={selectedIndex}
      emptyMessage="No jumps"
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
