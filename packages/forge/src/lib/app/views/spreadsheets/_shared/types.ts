/**
 * Shared types for VirtualTable-based spreadsheet views.
 */

export interface ColumnDefinition<T> {
  key: string;
  label: string;
  width?: string;
  sortable?: boolean;
  align?: 'left' | 'centre' | 'right';
  render?: (row: T) => string;
}

export interface SortState {
  column: string;
  direction: 'asc' | 'desc';
}

export interface VirtualTableState {
  sort: SortState | null;
  filterText: string;
}
