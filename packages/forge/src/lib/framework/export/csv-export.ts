/**
 * CSV export utility — generates RFC 4180 compliant CSV from table data.
 */

import type { ColumnDefinition } from '$lib/app/views/spreadsheets/_shared/types';

/** Escape a CSV field per RFC 4180: wrap in quotes if it contains comma, quote, or newline. */
function escapeField(value: string): string {
  if (value.includes(',') || value.includes('"') || value.includes('\n') || value.includes('\r')) {
    return `"${value.replace(/"/g, '""')}"`;
  }
  return value;
}

/** Get cell value from a row using column definition. */
function getCellValue<T>(row: T, column: ColumnDefinition<T>): string {
  if (column.render) return column.render(row);
  const value = (row as Record<string, unknown>)[column.key];
  return value == null ? '' : String(value);
}

/** Export table data as a CSV file download. */
export function exportTableToCSV<T>(
  columns: ColumnDefinition<T>[],
  rows: T[],
  filename: string,
): void {
  const headerLine = columns.map((c) => escapeField(c.label)).join(',');
  const dataLines = rows.map((row) =>
    columns.map((col) => escapeField(getCellValue(row, col))).join(','),
  );
  const csv = [headerLine, ...dataLines].join('\r\n');

  const blob = new Blob([csv], { type: 'text/csv;charset=utf-8' });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = filename.endsWith('.csv') ? filename : `${filename}.csv`;
  link.style.display = 'none';
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
  URL.revokeObjectURL(url);
}
