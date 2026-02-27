<script lang="ts">
  /**
   * DiagnosticSpreadsheet — table view of all diagnostics across files.
   */

  import { onMount, onDestroy, untrack } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { projectionRegistry } from '$lib/app/projections/ProjectionRegistry';
  import { navigationBroker } from '$lib/framework/navigation/NavigationBroker';
  import type { FileDiagnostics } from '$lib/app/projections/diagnostics-by-file';
  import type { Diagnostic } from '$lib/app/compiler/types';
  import type { ColumnDefinition, SortState, VirtualTableState } from './_shared/types';
  import VirtualTable from './_shared/VirtualTable.svelte';
  import FilterBar from './_shared/FilterBar.svelte';

  interface DiagnosticRow {
    severity: string;
    code: string;
    message: string;
    file: string;
    line: number;
  }

  interface Props {
    zoneId: string;
    zoneTypeId: string;
    state: VirtualTableState | null;
    onStateChange: (newState: unknown) => void;
  }

  let { zoneId, zoneTypeId, state: zoneState, onStateChange }: Props = $props();

  let rawRows: DiagnosticRow[] = $state([]);
  let sort: SortState | null = $state(untrack(() => zoneState?.sort ?? null));
  let filterText: string = $state(untrack(() => zoneState?.filterText ?? ''));
  let selectedIndex: number = $state(-1);

  const columns: ColumnDefinition<DiagnosticRow>[] = [
    { key: 'severity', label: 'Severity', width: '90px', render: (r) => severityIcon(r.severity) + ' ' + r.severity },
    { key: 'code', label: 'Code', width: '70px' },
    { key: 'message', label: 'Message', width: '360px' },
    { key: 'file', label: 'File', width: '180px' },
    { key: 'line', label: 'Line', width: '60px', align: 'right' },
  ];

  function severityIcon(severity: string): string {
    switch (severity) {
      case 'error': return '\u25CF';
      case 'warning': return '\u25B2';
      case 'info': return '\u25CB';
      default: return '\u25CB';
    }
  }

  let filteredRows = $derived.by(() => {
    let rows = rawRows;
    if (filterText) {
      const lower = filterText.toLowerCase();
      rows = rows.filter((r) =>
        r.message.toLowerCase().includes(lower) ||
        r.code.toLowerCase().includes(lower) ||
        r.severity.toLowerCase().includes(lower) ||
        r.file.toLowerCase().includes(lower)
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
    const fileDiags = projectionRegistry.get<FileDiagnostics[]>('urd.projection.diagnosticsByFile') ?? [];
    const rows: DiagnosticRow[] = [];
    for (const fd of fileDiags) {
      for (const diag of fd.diagnostics) {
        rows.push({
          severity: diag.severity,
          code: diag.code,
          message: diag.message,
          file: diag.span?.file ?? fd.file,
          line: diag.span?.startLine ?? 0,
        });
      }
    }
    rawRows = rows;
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

  function handleRowClick(_row: DiagnosticRow, index: number): void {
    selectedIndex = index;
  }

  function handleRowDoubleClick(row: DiagnosticRow): void {
    if (row.file && row.line) {
      navigationBroker.navigate({
        targetViewId: 'urd.codeEditor',
        params: { path: row.file, line: row.line },
      });
    }
  }
</script>

<div class="forge-spreadsheet-zone">
  <FilterBar {filterText} onFilterChange={handleFilterChange} placeholder="Filter diagnostics\u2026" />
  <div class="forge-spreadsheet-zone__table">
    <VirtualTable
      rows={filteredRows}
      {columns}
      {sort}
      selectedRowIndex={selectedIndex}
      emptyMessage="No diagnostics"
      rowKey={(_row, i) => String(i)}
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
