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
import { focusService } from '$lib/framework/focus/FocusService';
import { selectionContext } from '$lib/framework/selection/SelectionContext';
import { navigationBroker } from '$lib/framework/navigation/NavigationBroker';

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

  // 8. Focus tracking — click on a zone viewport to focus it
  const onZoneFocus = (e: PointerEvent) => {
    const target = e.target as HTMLElement;
    const viewport = target.closest('.forge-zone-viewport') as HTMLElement | null;
    if (!viewport) return;
    const zoneId = viewport.dataset.zoneId;
    const zoneType = viewport.dataset.zoneType;
    if (zoneId && zoneType) {
      focusService.focusZone(zoneId, zoneType);
    }
  };
  document.addEventListener('pointerdown', onZoneFocus);

  // Expose services on window for dev tools (non-production only)
  if (import.meta.env.DEV) {
    Object.assign(window, {
      __forge: { focusService, selectionContext, navigationBroker, bus, commandRegistry, viewRegistry },
    });
  }

  // 9. Selection containment — prevent text selection crossing zone boundaries
  //    On pointerdown inside a zone viewport, set user-select: none on all OTHER
  //    viewports via inline style. On pointerup, remove the inline styles.
  //    This is targeted (no global class toggles) and prevents the browser from
  //    extending a drag-selection into sibling zones.
  let lockedViewports: HTMLElement[] = [];

  const onPointerDown = (e: PointerEvent) => {
    if (e.button !== 0) return; // left click only
    const target = e.target as HTMLElement;
    const activeZone = target.closest('.forge-zone-viewport');
    if (!activeZone) return;

    // Disable selection on .forge-selectable elements in OTHER zone viewports.
    // Must target the selectable elements directly — a parent's user-select: none
    // doesn't override a child's explicit user-select: text.
    const allSelectables = document.querySelectorAll<HTMLElement>('.forge-zone-viewport .forge-selectable');
    for (const el of allSelectables) {
      if (!activeZone.contains(el)) {
        el.style.userSelect = 'none';
        el.style.webkitUserSelect = 'none';
        lockedViewports.push(el);
      }
    }
  };

  const onPointerUp = () => {
    for (const el of lockedViewports) {
      el.style.userSelect = '';
      el.style.webkitUserSelect = '';
    }
    lockedViewports = [];
  };

  document.addEventListener('pointerdown', onPointerDown);
  document.addEventListener('pointerup', onPointerUp);

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
    document.removeEventListener('pointerdown', onZoneFocus);
    document.removeEventListener('pointerdown', onPointerDown);
    document.removeEventListener('pointerup', onPointerUp);
  };
}
