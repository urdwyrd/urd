/**
 * Menu registry — contribution-based menu population.
 */

import type { MenuContribution } from '../types';

export type MenuId = 'file' | 'edit' | 'view' | 'window' | 'help';

const contributions = new Map<MenuId, MenuContribution[]>();

export function registerMenuContribution(contribution: MenuContribution): void {
  const list = contributions.get(contribution.menu) ?? [];
  list.push(contribution);
  contributions.set(contribution.menu, list);
}

export function getMenuContributions(menu: MenuId): MenuContribution[] {
  const items = contributions.get(menu) ?? [];
  // Sort by group, then by order within group
  return [...items].sort((a, b) => {
    if (a.group !== b.group) return a.group.localeCompare(b.group);
    return a.order - b.order;
  });
}

/** Group contributions by their group name, inserting separators between groups. */
export function getGroupedMenuContributions(menu: MenuId): MenuContribution[][] {
  const sorted = getMenuContributions(menu);
  const groups: MenuContribution[][] = [];
  let currentGroup: MenuContribution[] = [];
  let lastGroupName = '';

  for (const item of sorted) {
    if (item.group !== lastGroupName && currentGroup.length > 0) {
      groups.push(currentGroup);
      currentGroup = [];
    }
    currentGroup.push(item);
    lastGroupName = item.group;
  }
  if (currentGroup.length > 0) {
    groups.push(currentGroup);
  }

  return groups;
}

export function clearMenuContributions(): void {
  contributions.clear();
}
