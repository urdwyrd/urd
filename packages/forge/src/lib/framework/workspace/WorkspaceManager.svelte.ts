/**
 * Workspace manager — tabs, serialise/deserialise, corruption-safe load.
 */

import type { ZoneTree, ZoneTreeAction } from '../types';
import { zoneTreeReducer, createLeaf } from '../layout/ZoneTree';
import { ZoneStateStore } from '../layout/ZoneStateStore';
import { bus } from '../bus/MessageBus';
import { viewRegistry } from '../views/ViewRegistry';
import type { SerializedWorkspace, SerializedWorkspaceSet } from './types';

export interface Workspace {
  id: string;
  name: string;
  tree: ZoneTree;
  zoneStates: ZoneStateStore;
}

let nextWorkspaceId = 1;

function generateWorkspaceId(): string {
  return `ws_${(nextWorkspaceId++).toString(36)}`;
}

function createDefaultTree(): ZoneTree {
  return createLeaf('forge.placeholder.colour');
}

function createDefaultWorkspace(name = 'Default'): Workspace {
  return {
    id: generateWorkspaceId(),
    name,
    tree: createDefaultTree(),
    zoneStates: new ZoneStateStore(),
  };
}

export class WorkspaceManager {
  workspaces: Workspace[] = $state([]);
  activeIndex: number = $state(0);

  private templates = new Map<string, () => ZoneTree>();

  constructor() {
    this.workspaces = [createDefaultWorkspace()];
  }

  get active(): Workspace {
    return this.workspaces[this.activeIndex];
  }

  activate(index: number): void {
    if (index >= 0 && index < this.workspaces.length) {
      // Transfer singleton state (e.g. Code Editor tabs) to the target workspace
      const outgoing = this.workspaces[this.activeIndex];
      const incoming = this.workspaces[index];
      if (outgoing && incoming && outgoing !== incoming) {
        incoming.zoneStates.importSingletonStates(outgoing.zoneStates);
      }

      // Clear singleton ownership before the tree re-renders — avoids race
      // where new zones mount before old zones unmount their $effect cleanup.
      viewRegistry.clearActiveSingletons();

      this.activeIndex = index;
      if (bus.hasChannel('workspace.switched')) {
        bus.publish('workspace.switched', { index, workspaceId: this.workspaces[index].id });
      }
    }
  }

  create(name: string, tree?: ZoneTree): void {
    const ws: Workspace = {
      id: generateWorkspaceId(),
      name,
      tree: tree ?? createDefaultTree(),
      zoneStates: new ZoneStateStore(),
    };
    this.workspaces = [...this.workspaces, ws];
    this.activate(this.workspaces.length - 1);
  }

  duplicate(index: number): void {
    const source = this.workspaces[index];
    if (!source) return;
    const ws: Workspace = {
      id: generateWorkspaceId(),
      name: `${source.name} (copy)`,
      tree: structuredClone(source.tree),
      zoneStates: new ZoneStateStore(),
    };
    // Copy zone states
    ws.zoneStates.deserialize(source.zoneStates.serialize());
    this.workspaces = [...this.workspaces, ws];
    this.activate(this.workspaces.length - 1);
  }

  remove(index: number): void {
    if (this.workspaces.length <= 1) return; // always keep at least one
    this.workspaces = this.workspaces.filter((_, i) => i !== index);
    if (this.activeIndex >= this.workspaces.length) {
      this.activeIndex = this.workspaces.length - 1;
    }
  }

  rename(index: number, name: string): void {
    if (this.workspaces[index]) {
      this.workspaces[index].name = name;
      this.workspaces = [...this.workspaces]; // trigger reactivity
    }
  }

  /** Dispatch a tree action to the active workspace. */
  dispatch(action: ZoneTreeAction): void {
    const ws = this.active;
    ws.tree = zoneTreeReducer(ws.tree, action);
    this.workspaces = [...this.workspaces]; // trigger reactivity

    if (bus.hasChannel('layout.changed')) {
      bus.publish('layout.changed', { action, workspaceId: ws.id });
    }
  }

  registerTemplate(name: string, factory: () => ZoneTree): void {
    this.templates.set(name, factory);
  }

  createFromTemplate(name: string): void {
    const factory = this.templates.get(name);
    if (factory) {
      this.create(name, factory());
    }
  }

  serialize(): SerializedWorkspaceSet {
    return {
      version: 1,
      activeIndex: this.activeIndex,
      workspaces: this.workspaces.map((ws) => ({
        id: ws.id,
        name: ws.name,
        tree: ws.tree,
        zoneStates: ws.zoneStates.serialize(),
      })),
    };
  }

  /**
   * Corruption-safe deserialise.
   * If parsing or migration fails, preserves the broken data concept
   * and initialises with defaults.
   */
  deserialize(data: SerializedWorkspaceSet): boolean {
    try {
      if (!data || !Array.isArray(data.workspaces) || data.workspaces.length === 0) {
        throw new Error('Invalid workspace data');
      }

      this.workspaces = data.workspaces.map((ws: SerializedWorkspace) => {
        const store = new ZoneStateStore();
        if (ws.zoneStates) {
          store.deserialize(ws.zoneStates);
        }
        return {
          id: ws.id,
          name: ws.name,
          tree: ws.tree,
          zoneStates: store,
        };
      });

      this.activeIndex = Math.min(data.activeIndex ?? 0, this.workspaces.length - 1);
      return true;
    } catch (err) {
      console.warn('Workspace deserialisation failed — loading defaults:', err);
      this.workspaces = [createDefaultWorkspace()];
      this.activeIndex = 0;
      return false;
    }
  }
}

/** Singleton workspace manager. */
export const workspaceManager = new WorkspaceManager();
