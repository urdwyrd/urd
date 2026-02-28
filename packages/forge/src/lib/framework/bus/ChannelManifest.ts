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
  bus.registerChannel({ id: 'settings.projectOverride.changed', domain: 'settings', retainLast: true });

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
  bus.registerChannel({ id: 'workspace.stateChanged', domain: 'workspace', retainLast: false });

  // Command execution log
  bus.registerChannel({ id: 'command.executed', domain: 'commands', retainLast: false });

  // Project lifecycle
  bus.registerChannel({ id: 'project.opened', domain: 'project', retainLast: true });
  bus.registerChannel({ id: 'project.closed', domain: 'project', retainLast: false });

  // Selection
  bus.registerChannel({ id: 'selection.primary', domain: 'selection', retainLast: true });

  // Filter
  bus.registerChannel({ id: 'filter.changed', domain: 'filter', retainLast: true });

  // Compiler lifecycle (registered at framework level so bus monitor can observe)
  bus.registerChannel({ id: 'compiler.started', domain: 'compiler', retainLast: false });
  bus.registerChannel({ id: 'compiler.completed', domain: 'compiler', retainLast: true });
  bus.registerChannel({ id: 'compiler.error', domain: 'compiler', retainLast: false });

  // Projections
  bus.registerChannel({ id: 'projection.updated', domain: 'projections', retainLast: false });

  // Navigation — carries resolved intent params to the target zone component
  bus.registerChannel({ id: 'navigation.completed', domain: 'navigation', retainLast: false });

  // Editor state
  bus.registerChannel({ id: 'editor.activeFile', domain: 'editor', retainLast: true });

  // Editor commands (signals dispatched from global keybindings to the active editor)
  bus.registerChannel({ id: 'editor.closeTab', domain: 'editor', retainLast: false });
  bus.registerChannel({ id: 'editor.nextTab', domain: 'editor', retainLast: false });
  bus.registerChannel({ id: 'editor.prevTab', domain: 'editor', retainLast: false });
  bus.registerChannel({ id: 'editor.save', domain: 'editor', retainLast: false });

  // Playback / runtime
  bus.registerChannel({ id: 'playback.state.changed', domain: 'playback', retainLast: true });
  bus.registerChannel({ id: 'playback.event', domain: 'playback', retainLast: false });

  // Coverage
  bus.registerChannel({ id: 'coverage.overlay.updated', domain: 'coverage', retainLast: true });
  bus.registerChannel({ id: 'coverage.overlay.cleared', domain: 'coverage', retainLast: false });

  // Analysis
  bus.registerChannel({ id: 'analysis.monteCarlo.progress', domain: 'analysis', retainLast: true });
  bus.registerChannel({ id: 'analysis.monteCarlo.completed', domain: 'analysis', retainLast: true });
  bus.registerChannel({ id: 'analysis.baseline.saved', domain: 'analysis', retainLast: true });
}
