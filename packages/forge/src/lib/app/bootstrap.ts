/**
 * Application bootstrap — registers all framework views, commands, and menus.
 * Phase 1: only placeholders and framework commands.
 */

import { viewRegistry } from '$lib/framework/views/ViewRegistry';
import { commandRegistry } from '$lib/framework/commands/CommandRegistry';
import { registerMenuContribution } from '$lib/framework/menu/MenuRegistry';
import { registerFrameworkChannels } from '$lib/framework/bus/ChannelManifest';
import { bus } from '$lib/framework/bus/MessageBus';
import { appSettings } from '$lib/framework/settings/AppSettingsService';
import { initTheme, toggleTheme, getCurrentTheme } from '$lib/framework/theme/ThemeEngine';
import { installKeybindingManager } from '$lib/framework/commands/KeybindingManager';
import { projectManager } from '$lib/framework/project/ProjectManager.svelte';
import { workspaceManager } from '$lib/framework/workspace/WorkspaceManager.svelte';

export async function bootstrap(): Promise<() => void> {
  // 1. Register framework bus channels
  registerFrameworkChannels(bus);

  // 2. Load settings
  await appSettings.load();

  // 3. Initialise theme from saved settings
  initTheme(appSettings.get('theme'));

  // 4. Register placeholder views
  viewRegistry.register({
    id: 'forge.placeholder.colour',
    name: 'Colour',
    icon: 'colour',
    category: 'Debug',
    component: () => import('$lib/framework/placeholders/PlaceholderColour.svelte'),
    requiresProject: false,
    stateVersion: 1,
    defaultState: null,
  });

  viewRegistry.register({
    id: 'forge.placeholder.info',
    name: 'Info',
    icon: 'info',
    category: 'Debug',
    component: () => import('$lib/framework/placeholders/PlaceholderInfo.svelte'),
    requiresProject: false,
    stateVersion: 1,
    defaultState: null,
  });

  viewRegistry.register({
    id: 'forge.placeholder.busMonitor',
    name: 'Bus Monitor',
    icon: 'monitor',
    category: 'Debug',
    component: () => import('$lib/framework/placeholders/PlaceholderBusMonitor.svelte'),
    requiresProject: false,
    stateVersion: 1,
    defaultState: null,
  });

  viewRegistry.register({
    id: 'forge.placeholder.commandLog',
    name: 'Command Log',
    icon: 'log',
    category: 'Debug',
    component: () => import('$lib/framework/placeholders/PlaceholderCommandLog.svelte'),
    requiresProject: false,
    stateVersion: 1,
    defaultState: null,
  });

  // 5. Register framework commands
  commandRegistry.register({
    id: 'forge.window.toggleFullscreen',
    title: 'Toggle Fullscreen',
    category: 'Window',
    keybinding: 'f11',
    globalWhenEditorFocused: true,
    execute: async () => {
      if ('__TAURI_INTERNALS__' in window) {
        const { getCurrentWindow } = await import('@tauri-apps/api/window');
        const win = getCurrentWindow();
        const isFullscreen = await win.isFullscreen();
        await win.setFullscreen(!isFullscreen);
      } else {
        if (document.fullscreenElement) {
          document.exitFullscreen();
        } else {
          document.documentElement.requestFullscreen();
        }
      }
      return null;
    },
  });

  commandRegistry.register({
    id: 'forge.theme.toggle',
    title: 'Toggle Theme',
    category: 'View',
    keybinding: 'ctrl+shift+t',
    execute: () => {
      toggleTheme();
      appSettings.set('theme', getCurrentTheme());
      return null;
    },
  });

  commandRegistry.register({
    id: 'forge.project.open',
    title: 'Open Project',
    category: 'File',
    keybinding: 'ctrl+o',
    execute: async () => {
      try {
        const { open } = await import('@tauri-apps/plugin-dialog');
        const selected = await open({
          directory: true,
          title: 'Open Urd Project',
        });
        if (selected) {
          await projectManager.openPath(selected as string);
        }
      } catch {
        // Browser dev mode fallback — open a mock project
        await projectManager.openPath('/mock/urd-project');
      }
      return null;
    },
  });

  commandRegistry.register({
    id: 'forge.project.close',
    title: 'Close Project',
    category: 'File',
    execute: () => {
      projectManager.close();
      return null;
    },
  });

  commandRegistry.register({
    id: 'forge.app.quit',
    title: 'Quit',
    category: 'File',
    keybinding: 'ctrl+q',
    execute: async () => {
      if ('__TAURI_INTERNALS__' in window) {
        const { getCurrentWindow } = await import('@tauri-apps/api/window');
        await getCurrentWindow().close();
      }
      // Browser dev mode: window.close() only works for script-opened windows, so no-op
      return null;
    },
  });

  commandRegistry.register({
    id: 'forge.edit.commandPalette',
    title: 'Command Palette',
    category: 'Edit',
    keybinding: 'ctrl+shift+p',
    globalWhenEditorFocused: true,
    execute: () => {
      // Stub — will be implemented in a later phase
      console.log('Command Palette opened (stub)');
      return null;
    },
  });

  // 5b. Zone operation commands (used by context menus)
  commandRegistry.register({
    id: 'forge.zone.splitHorizontal',
    title: 'Split Left / Right',
    category: 'Layout',
    execute: (...args: unknown[]) => {
      const opts = args[0] as { zoneId: string } | undefined;
      if (opts?.zoneId) {
        workspaceManager.dispatch({ type: 'split', zoneId: opts.zoneId, direction: 'horizontal' });
      }
      return null;
    },
  });

  commandRegistry.register({
    id: 'forge.zone.splitVertical',
    title: 'Split Top / Bottom',
    category: 'Layout',
    execute: (...args: unknown[]) => {
      const opts = args[0] as { zoneId: string } | undefined;
      if (opts?.zoneId) {
        workspaceManager.dispatch({ type: 'split', zoneId: opts.zoneId, direction: 'vertical' });
      }
      return null;
    },
  });

  commandRegistry.register({
    id: 'forge.zone.joinFirst',
    title: 'Join (Keep Left / Top)',
    category: 'Layout',
    execute: (...args: unknown[]) => {
      const opts = args[0] as { dividerId: string } | undefined;
      if (opts?.dividerId) {
        workspaceManager.dispatch({ type: 'join', dividerId: opts.dividerId, keep: 'first' });
      }
      return null;
    },
  });

  commandRegistry.register({
    id: 'forge.zone.joinSecond',
    title: 'Join (Keep Right / Bottom)',
    category: 'Layout',
    execute: (...args: unknown[]) => {
      const opts = args[0] as { dividerId: string } | undefined;
      if (opts?.dividerId) {
        workspaceManager.dispatch({ type: 'join', dividerId: opts.dividerId, keep: 'second' });
      }
      return null;
    },
  });

  commandRegistry.register({
    id: 'forge.zone.swap',
    title: 'Swap Zones',
    category: 'Layout',
    execute: (...args: unknown[]) => {
      const opts = args[0] as { dividerId: string } | undefined;
      if (opts?.dividerId) {
        workspaceManager.dispatch({ type: 'swap', dividerId: opts.dividerId });
      }
      return null;
    },
  });

  commandRegistry.register({
    id: 'forge.zone.resetDivider',
    title: 'Reset Divider',
    category: 'Layout',
    execute: (...args: unknown[]) => {
      const opts = args[0] as { dividerId: string } | undefined;
      if (opts?.dividerId) {
        workspaceManager.dispatch({ type: 'resetDivider', dividerId: opts.dividerId });
      }
      return null;
    },
  });

  // 6. Register menu contributions
  registerMenuContribution({ menu: 'file', group: 'project', order: 1, commandId: 'forge.project.open', label: 'Open Project' });
  registerMenuContribution({ menu: 'file', group: 'project', order: 2, commandId: 'forge.project.close', label: 'Close Project' });
  registerMenuContribution({ menu: 'file', group: 'quit', order: 99, commandId: 'forge.app.quit', label: 'Quit' });

  registerMenuContribution({ menu: 'edit', group: 'palette', order: 1, commandId: 'forge.edit.commandPalette', label: 'Command Palette' });

  registerMenuContribution({ menu: 'view', group: 'fullscreen', order: 1, commandId: 'forge.window.toggleFullscreen', label: 'Toggle Fullscreen' });
  registerMenuContribution({ menu: 'view', group: 'theme', order: 2, commandId: 'forge.theme.toggle', label: 'Toggle Theme' });

  // 7. Install keyboard sovereignty
  const removeKeybindings = installKeybindingManager();

  // 8. Selection containment — prevent text selection crossing zone boundaries
  const onSelectionChange = () => {
    const sel = document.getSelection();
    if (!sel || sel.rangeCount === 0 || sel.isCollapsed) return;

    const anchorEl = sel.anchorNode instanceof HTMLElement ? sel.anchorNode : sel.anchorNode?.parentElement;
    const focusEl = sel.focusNode instanceof HTMLElement ? sel.focusNode : sel.focusNode?.parentElement;
    if (!anchorEl || !focusEl) return;

    const anchorZone = anchorEl.closest('.forge-zone-viewport');
    const focusZone = focusEl.closest('.forge-zone-viewport');

    if (anchorZone && focusZone && anchorZone !== focusZone) {
      sel.collapseToStart();
    }
  };
  document.addEventListener('selectionchange', onSelectionChange);

  // 9. Global error handling
  window.onerror = (message, source, line, col, error) => {
    console.error('Global error:', { message, source, line, col, error });
    if (bus.hasChannel('system.error')) {
      bus.publish('system.error', {
        source: 'window.onerror',
        message: String(message),
        error: error?.message,
      });
    }
  };

  window.onunhandledrejection = (event) => {
    console.error('Unhandled rejection:', event.reason);
    if (bus.hasChannel('system.error')) {
      bus.publish('system.error', {
        source: 'unhandledrejection',
        message: String(event.reason),
      });
    }
  };

  // Return cleanup function
  return () => {
    removeKeybindings();
    document.removeEventListener('selectionchange', onSelectionChange);
  };
}
