/**
 * Context menu provider registry.
 * Framework code — no Urd-specific imports.
 */

import type { ContextMenuItem, ContextMenuTarget } from '../types';

export interface ContextMenuProvider {
  getItems(target: ContextMenuTarget): ContextMenuItem[] | null;
}

const providers: ContextMenuProvider[] = [];

export function registerContextMenuProvider(provider: ContextMenuProvider): void {
  providers.push(provider);
}

/** Collect items from all providers for a given target. */
export function getContextMenuItems(target: ContextMenuTarget): ContextMenuItem[] {
  const items: ContextMenuItem[] = [];
  for (const provider of providers) {
    const result = provider.getItems(target);
    if (result) {
      if (items.length > 0) {
        items.push({ label: '', commandId: '', separator: true });
      }
      items.push(...result);
    }
  }
  return items;
}
