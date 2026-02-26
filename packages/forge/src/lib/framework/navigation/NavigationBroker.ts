/**
 * Navigation broker — resolves NavigationIntents against the view registry.
 *
 * For Phase 2, intents targeting views that don't exist yet (e.g., urd.codeEditor)
 * are queued. The broker handles singleton/singleton-autocreate/multi strategies
 * by consulting the view registry and workspace manager.
 */

import type { NavigationIntent } from '../types';
import { viewRegistry } from '../views/ViewRegistry';
import { workspaceManager } from '../workspace/WorkspaceManager.svelte';
import { focusService } from '../focus/FocusService.svelte';
import { collectLeaves } from '../layout/ZoneTree';

export type NavigationResult =
  | { resolved: true; zoneId: string }
  | { resolved: false; reason: string };

export class NavigationBroker {
  private queue: NavigationIntent[] = [];

  /**
   * Navigate to a view. Resolves the intent against the view registry's
   * navigation strategy and the current workspace layout.
   */
  navigate(intent: NavigationIntent): NavigationResult {
    const view = viewRegistry.get(intent.targetViewId);
    if (!view) {
      // Queue for later — the view type may not be registered yet
      this.queue.push(intent);
      return { resolved: false, reason: `View not registered: ${intent.targetViewId}` };
    }

    const strategy = view.navigationStrategy ?? 'multi';

    switch (strategy) {
      case 'singleton': {
        // Find an existing instance or fail
        const existingZoneId = this.findZoneWithType(intent.targetViewId);
        if (existingZoneId) {
          focusService.focusZone(existingZoneId, intent.targetViewId);
          return { resolved: true, zoneId: existingZoneId };
        }
        return { resolved: false, reason: `No existing instance of singleton view: ${intent.targetViewId}` };
      }

      case 'singleton-autocreate': {
        // Find an existing instance or create one in the active zone
        const existingZoneId = this.findZoneWithType(intent.targetViewId);
        if (existingZoneId) {
          focusService.focusZone(existingZoneId, intent.targetViewId);
          return { resolved: true, zoneId: existingZoneId };
        }
        // Auto-create: change the focused zone's type (or the preferred zone)
        const targetZoneId = intent.preferZoneId ?? focusService.activeZoneId;
        if (targetZoneId) {
          workspaceManager.dispatch({
            type: 'changeType',
            zoneId: targetZoneId,
            newTypeId: intent.targetViewId,
          });
          focusService.focusZone(targetZoneId, intent.targetViewId);
          return { resolved: true, zoneId: targetZoneId };
        }
        return { resolved: false, reason: 'No zone available for singleton-autocreate' };
      }

      case 'multi': {
        // Open in the preferred zone or the focused zone
        const targetZoneId = intent.preferZoneId ?? focusService.activeZoneId;
        if (targetZoneId) {
          workspaceManager.dispatch({
            type: 'changeType',
            zoneId: targetZoneId,
            newTypeId: intent.targetViewId,
          });
          focusService.focusZone(targetZoneId, intent.targetViewId);
          return { resolved: true, zoneId: targetZoneId };
        }
        return { resolved: false, reason: 'No zone available for multi view' };
      }
    }
  }

  /** Drain the intent queue, attempting to resolve each. Returns unresolved intents. */
  drainQueue(): NavigationIntent[] {
    const pending = [...this.queue];
    this.queue = [];
    const stillPending: NavigationIntent[] = [];
    for (const intent of pending) {
      const result = this.navigate(intent);
      if (!result.resolved) {
        stillPending.push(intent);
      }
    }
    this.queue = stillPending;
    return stillPending;
  }

  /** Returns the number of queued intents. */
  get queueSize(): number {
    return this.queue.length;
  }

  /** Find a zone in the active workspace tree that has the given view type. */
  private findZoneWithType(viewTypeId: string): string | null {
    const tree = workspaceManager.active.tree;
    const leaves = collectLeaves(tree);
    const match = leaves.find((leaf) => leaf.zoneTypeId === viewTypeId);
    return match?.id ?? null;
  }
}

/** Singleton navigation broker. */
export const navigationBroker = new NavigationBroker();
