/**
 * Shared framework types.
 */

// ===== BSP Zone Tree =====

export type ZoneTree = SplitNode | LeafNode;

export interface SplitNode {
  kind: 'split';
  id: string;
  direction: 'horizontal' | 'vertical';
  ratio: number; // 0.0–1.0, position of the divider
  children: [ZoneTree, ZoneTree];
}

export interface LeafNode {
  kind: 'leaf';
  id: string; // unique zone ID (e.g., "zone_a3f2")
  zoneTypeId: string; // registry key (e.g., "forge.placeholder.colour")
  singletonRef?: true; // if true, mounts shared singleton instance
}

// ===== Zone Tree Actions =====

export type ZoneTreeAction =
  | { type: 'split'; zoneId: string; direction: 'horizontal' | 'vertical' }
  | { type: 'join'; dividerId: string; keep: 'first' | 'second' }
  | { type: 'swap'; dividerId: string }
  | { type: 'resize'; dividerId: string; ratio: number }
  | { type: 'changeType'; zoneId: string; newTypeId: string }
  | { type: 'resetDivider'; dividerId: string };

// ===== Zone State =====

export interface PersistedZoneState {
  stateVersion: number;
  data: unknown;
}

// ===== Context Menu =====

export interface ContextMenuItem {
  label: string;
  commandId: string;
  commandArgs?: Record<string, unknown>;
  icon?: string;
  keybinding?: string;
  disabled?: boolean;
  separator?: boolean;
  children?: ContextMenuItem[];
}

export interface ContextMenuTarget {
  zoneId: string | null;
  zoneType: string | null;
  element: HTMLElement;
  data?: Record<string, unknown>;
}

// ===== Menu Bar =====

export interface MenuContribution {
  menu: 'file' | 'edit' | 'view' | 'window' | 'help';
  group: string;
  order: number;
  commandId: string;
  label?: string;
  submenu?: MenuContribution[];
}
