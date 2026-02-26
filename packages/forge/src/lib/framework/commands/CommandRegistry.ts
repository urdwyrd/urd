/**
 * Command registry — every user-visible action is a command.
 * Commands return UndoAction hooks for undo support.
 */

import { bus } from '../bus/MessageBus';

export type UndoAction = (() => void | Promise<void>) | null;

export interface CommandDefinition {
  id: string;
  title: string;
  category: string;
  keybinding?: string;
  /** If true, this command fires even when a CodeMirror editor has focus. */
  globalWhenEditorFocused?: boolean;
  execute: (...args: unknown[]) => UndoAction | Promise<UndoAction> | void | Promise<void>;
}

export interface CommandExecution {
  commandId: string;
  timestamp: number;
  args: unknown[];
}

export class CommandRegistry {
  private commands = new Map<string, CommandDefinition>();
  private keybindingMap = new Map<string, string>(); // normalised key -> command ID

  register(command: CommandDefinition): void {
    if (this.commands.has(command.id)) {
      console.warn(`Command already registered: ${command.id} — overwriting.`);
    }
    this.commands.set(command.id, command);
    if (command.keybinding) {
      this.keybindingMap.set(normaliseKeybinding(command.keybinding), command.id);
    }
  }

  unregister(commandId: string): void {
    const cmd = this.commands.get(commandId);
    if (cmd?.keybinding) {
      this.keybindingMap.delete(normaliseKeybinding(cmd.keybinding));
    }
    this.commands.delete(commandId);
  }

  get(commandId: string): CommandDefinition | undefined {
    return this.commands.get(commandId);
  }

  list(): CommandDefinition[] {
    return Array.from(this.commands.values());
  }

  listByCategory(): Map<string, CommandDefinition[]> {
    const grouped = new Map<string, CommandDefinition[]>();
    for (const cmd of this.commands.values()) {
      const list = grouped.get(cmd.category) ?? [];
      list.push(cmd);
      grouped.set(cmd.category, list);
    }
    return grouped;
  }

  resolveByKeybinding(key: string): CommandDefinition | undefined {
    const commandId = this.keybindingMap.get(normaliseKeybinding(key));
    return commandId ? this.commands.get(commandId) : undefined;
  }

  async execute(commandId: string, ...args: unknown[]): Promise<UndoAction | void> {
    const cmd = this.commands.get(commandId);
    if (!cmd) {
      console.warn(`Command not found: ${commandId}`);
      return;
    }

    try {
      const result = await cmd.execute(...args);

      // Publish execution to bus for PlaceholderCommandLog
      if (bus.hasChannel('command.executed')) {
        bus.publish('command.executed', {
          commandId,
          timestamp: Date.now(),
          args,
        } satisfies CommandExecution);
      }

      return result;
    } catch (err) {
      console.error(`Command execution failed: ${commandId}`, err);
      if (bus.hasChannel('system.error')) {
        bus.publish('system.error', {
          source: 'command',
          commandId,
          error: err instanceof Error ? err.message : String(err),
        });
      }
    }
  }

  async executeByKeybinding(key: string): Promise<void> {
    const cmd = this.resolveByKeybinding(key);
    if (cmd) {
      await this.execute(cmd.id);
    }
  }

  /** Update a command's keybinding at runtime (for user overrides). */
  rebind(commandId: string, newKeybinding: string | null): void {
    const cmd = this.commands.get(commandId);
    if (!cmd) return;

    // Remove old binding
    if (cmd.keybinding) {
      this.keybindingMap.delete(normaliseKeybinding(cmd.keybinding));
    }

    // Set new binding
    if (newKeybinding) {
      cmd.keybinding = newKeybinding;
      this.keybindingMap.set(normaliseKeybinding(newKeybinding), commandId);
    } else {
      cmd.keybinding = undefined;
    }
  }
}

/** Normalise a keybinding string to a canonical form for lookup. */
export function normaliseKeybinding(key: string): string {
  return key
    .toLowerCase()
    .split('+')
    .map((part) => part.trim())
    .sort((a, b) => {
      // Modifiers first, in canonical order
      const order = ['ctrl', 'alt', 'shift', 'meta'];
      const ai = order.indexOf(a);
      const bi = order.indexOf(b);
      if (ai !== -1 && bi !== -1) return ai - bi;
      if (ai !== -1) return -1;
      if (bi !== -1) return 1;
      return a.localeCompare(b);
    })
    .join('+');
}

/** Format a KeyboardEvent into a normalised keybinding string. */
export function formatKeybinding(e: KeyboardEvent): string {
  const parts: string[] = [];
  if (e.ctrlKey) parts.push('ctrl');
  if (e.altKey) parts.push('alt');
  if (e.shiftKey) parts.push('shift');
  if (e.metaKey) parts.push('meta');

  const key = e.key.toLowerCase();
  // Avoid duplicating modifier keys
  if (!['control', 'alt', 'shift', 'meta'].includes(key)) {
    parts.push(key);
  }

  return parts.join('+');
}

/** Singleton command registry. */
export const commandRegistry = new CommandRegistry();
