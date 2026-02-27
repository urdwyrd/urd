/**
 * Zone state store — single authority for all zone state.
 *
 * Key format: zone:${zoneId}::type:${zoneTypeId}
 * State is versioned to prevent bricked workspaces as views evolve.
 */

import type { PersistedZoneState } from '../types';
import { viewRegistry } from '../views/ViewRegistry';

export class ZoneStateStore {
  private store = new Map<string, PersistedZoneState>();

  private makeKey(zoneId: string, zoneTypeId: string): string {
    return `zone:${zoneId}::type:${zoneTypeId}`;
  }

  /**
   * Returns the persisted state, running migration if stateVersion is stale.
   * If migration fails or no state exists, returns the view's defaultState.
   */
  get(zoneId: string, zoneTypeId: string): unknown {
    const key = this.makeKey(zoneId, zoneTypeId);
    const entry = this.store.get(key);
    const view = viewRegistry.get(zoneTypeId);

    if (!entry || !view) {
      return view?.defaultState ?? null;
    }

    if (entry.stateVersion === view.stateVersion) {
      return entry.data;
    }

    // State version mismatch — try migration
    if (view.migrateState) {
      try {
        const migrated = view.migrateState(entry.data, entry.stateVersion);
        if (migrated !== undefined) {
          this.store.set(key, { stateVersion: view.stateVersion, data: migrated });
          return migrated;
        }
      } catch (err) {
        console.warn(`State migration failed for ${key}:`, err);
      }
    }

    // Migration failed or unavailable — return default
    return view.defaultState;
  }

  /**
   * Persists zone state with current stateVersion.
   * Called by the zone component on state changes.
   */
  set(zoneId: string, zoneTypeId: string, state: unknown): void {
    const key = this.makeKey(zoneId, zoneTypeId);
    const view = viewRegistry.get(zoneTypeId);
    this.store.set(key, {
      stateVersion: view?.stateVersion ?? 1,
      data: state,
    });
  }

  /** Returns singleton state shared across all zones of a given singleton view type. */
  getSingletonState(zoneTypeId: string): unknown {
    const key = `singleton::${zoneTypeId}`;
    const entry = this.store.get(key);
    const view = viewRegistry.get(zoneTypeId);

    if (!entry || !view) {
      return view?.defaultState ?? null;
    }

    if (entry.stateVersion === view.stateVersion) {
      return entry.data;
    }

    // State version mismatch — try migration
    if (view.migrateState) {
      try {
        const migrated = view.migrateState(entry.data, entry.stateVersion);
        if (migrated !== undefined) {
          this.store.set(key, { stateVersion: view.stateVersion, data: migrated });
          return migrated;
        }
      } catch (err) {
        console.warn(`Singleton state migration failed for ${key}:`, err);
      }
    }

    return view.defaultState;
  }

  /** Persists singleton state shared across all zones of a given singleton view type. */
  setSingletonState(zoneTypeId: string, state: unknown): void {
    const key = `singleton::${zoneTypeId}`;
    const view = viewRegistry.get(zoneTypeId);
    this.store.set(key, {
      stateVersion: view?.stateVersion ?? 1,
      data: state,
    });
  }

  /** Copy all singleton:: entries from another store into this one. */
  importSingletonStates(source: ZoneStateStore): void {
    for (const [key, value] of source.store) {
      if (key.startsWith('singleton::')) {
        this.store.set(key, value);
      }
    }
  }

  /** Clears all state for a zone when it is destroyed (via merge). */
  purge(zoneId: string): void {
    const prefix = `zone:${zoneId}::`;
    for (const key of this.store.keys()) {
      if (key.startsWith(prefix)) {
        this.store.delete(key);
      }
    }
  }

  /** Serialise the entire store for workspace persistence. */
  serialize(): Record<string, PersistedZoneState> {
    const result: Record<string, PersistedZoneState> = {};
    for (const [key, value] of this.store) {
      result[key] = value;
    }
    return result;
  }

  /** Restore from serialised data, running migrations as needed. */
  deserialize(data: Record<string, PersistedZoneState>): void {
    this.store.clear();
    for (const [key, value] of Object.entries(data)) {
      this.store.set(key, value);
    }
  }

  /** Clear all state. */
  clear(): void {
    this.store.clear();
  }
}
