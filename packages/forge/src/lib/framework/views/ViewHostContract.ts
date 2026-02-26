/**
 * View host contract — interfaces that views optionally implement
 * to contribute toolbar items and status hints.
 */

export interface ViewToolbarItem {
  id: string;
  icon: string;
  label: string;
  command: string;
  commandArgs?: Record<string, unknown>;
  active?: boolean;
  disabled?: boolean;
}

export interface ViewToolbarProvider {
  getToolbarItems(): ViewToolbarItem[];
}

export interface ViewStatusProvider {
  getStatusHint(): string | null;
}
