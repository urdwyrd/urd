/**
 * Framework-level bus channel definitions.
 * Application-specific channels are registered in app/compiler/channels.ts.
 */

import type { MessageBus } from './MessageBus';

export function registerFrameworkChannels(bus: MessageBus): void {
  // System-level error reporting
  bus.registerChannel({ id: 'system.error', domain: 'system', retainLast: false });

  // Settings changes (retainLast so late subscribers see current state)
  bus.registerChannel({ id: 'settings.changed', domain: 'settings', retainLast: true });

  // Theme changes
  bus.registerChannel({ id: 'theme.changed', domain: 'theme', retainLast: true });

  // Zone lifecycle events
  bus.registerChannel({ id: 'zone.created', domain: 'layout', retainLast: false });
  bus.registerChannel({ id: 'zone.destroyed', domain: 'layout', retainLast: false });
  bus.registerChannel({ id: 'zone.typeChanged', domain: 'layout', retainLast: false });
  bus.registerChannel({ id: 'zone.error', domain: 'layout', retainLast: false });

  // Layout mutations
  bus.registerChannel({ id: 'layout.changed', domain: 'layout', retainLast: true });

  // Focus tracking
  bus.registerChannel({ id: 'focus.zoneChanged', domain: 'focus', retainLast: true });

  // Workspace events
  bus.registerChannel({ id: 'workspace.switched', domain: 'workspace', retainLast: true });

  // Command execution log
  bus.registerChannel({ id: 'command.executed', domain: 'commands', retainLast: false });

  // Project lifecycle
  bus.registerChannel({ id: 'project.opened', domain: 'project', retainLast: true });
  bus.registerChannel({ id: 'project.closed', domain: 'project', retainLast: false });
}
