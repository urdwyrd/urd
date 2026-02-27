/**
 * View registry — register, list, lazy load views by ID.
 * Framework code. No Urd-specific imports.
 */

import type { Component } from 'svelte';

export type NavigationStrategy = 'singleton-autocreate' | 'singleton' | 'multi';

export interface ViewCapabilities {
  supportsSelection?: boolean;
  supportsFilter?: boolean;
  supportsNavigationTarget?: boolean;
  exportFormats?: string[];
}

export interface ViewRegistration {
  id: string;
  name: string;
  icon: string;
  category: string;
  component: () => Promise<{ default: Component }>;

  navigationStrategy?: NavigationStrategy;

  /** If true (default), hidden from zone type dropdown when no project is open. */
  requiresProject?: boolean;

  capabilities?: ViewCapabilities;

  stateVersion: number;
  defaultState: unknown;
  migrateState?: (oldState: unknown, fromVersion: number) => unknown;
}

export class ViewRegistry {
  private views = new Map<string, ViewRegistration>();
  private activeSingletons = new Map<string, string>(); // viewTypeId → ownerZoneId

  register(view: ViewRegistration): void {
    if (this.views.has(view.id)) {
      console.warn(`View already registered: ${view.id} — overwriting.`);
    }
    this.views.set(view.id, view);
  }

  get(id: string): ViewRegistration | undefined {
    return this.views.get(id);
  }

  list(): ViewRegistration[] {
    return Array.from(this.views.values());
  }

  /** List views grouped by category, sorted alphabetically within each group. */
  listByCategory(): Map<string, ViewRegistration[]> {
    const grouped = new Map<string, ViewRegistration[]>();
    for (const view of this.views.values()) {
      const list = grouped.get(view.category) ?? [];
      list.push(view);
      grouped.set(view.category, list);
    }
    for (const list of grouped.values()) {
      list.sort((a, b) => a.name.localeCompare(b.name));
    }
    return grouped;
  }

  /** Returns views available given the current project state. */
  listAvailable(projectOpen: boolean): ViewRegistration[] {
    return this.list().filter((v) => {
      if (v.requiresProject !== false && !projectOpen) return false;
      return true;
    });
  }

  isSingletonActive(id: string): boolean {
    return this.activeSingletons.has(id);
  }

  getSingletonZoneId(id: string): string | null {
    return this.activeSingletons.get(id) ?? null;
  }

  markSingletonActive(id: string, zoneId: string): void {
    this.activeSingletons.set(id, zoneId);
  }

  markSingletonInactive(id: string): void {
    this.activeSingletons.delete(id);
  }

  /** Release all singleton ownership. Called on workspace switch before new zones mount. */
  clearActiveSingletons(): void {
    this.activeSingletons.clear();
  }

  isSingleton(id: string): boolean {
    const view = this.views.get(id);
    return view?.navigationStrategy === 'singleton' ||
      view?.navigationStrategy === 'singleton-autocreate';
  }
}

/** Singleton view registry. */
export const viewRegistry = new ViewRegistry();
