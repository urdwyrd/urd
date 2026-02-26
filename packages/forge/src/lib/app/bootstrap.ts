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
import { focusService } from '$lib/framework/focus/FocusService.svelte';
import { selectionContext } from '$lib/framework/selection/SelectionContext';
import { navigationBroker } from '$lib/framework/navigation/NavigationBroker';
import { activeDialog } from '$lib/framework/layout/ActiveDialog.svelte';
import { bufferMap } from '$lib/app/compiler/BufferMap';
import { TauriFileSystem } from '$lib/app/filesystem/TauriFileSystem';
import { MemoryFileSystem } from '$lib/app/filesystem/MemoryFileSystem';
import type { ForgeFileSystem } from '$lib/framework/filesystem/FileSystem';
import { CompilerOutputCache } from '$lib/app/compiler/CompilerOutputCache';
import { MockCompilerService } from '$lib/app/compiler/MockCompilerService';
import { TauriCompiler } from '$lib/app/compiler/TauriCompiler';
import { RecompilePipeline } from '$lib/app/compiler/RecompilePipeline';
import { projectionRegistry } from '$lib/app/projections/ProjectionRegistry';
import { entityTableProjection } from '$lib/app/projections/entity-table';
import { locationGraphProjection } from '$lib/app/projections/location-graph';
import { diagnosticsByFileProjection } from '$lib/app/projections/diagnostics-by-file';
import type { CompilerService } from '$lib/app/compiler/types';

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

  // 4b. Settings/keybindings open as floating dialogs (not zone views)

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
      if (focusService.mode === 'commandPalette') {
        focusService.popMode();
      } else {
        focusService.pushMode('commandPalette');
      }
      return null;
    },
  });

  commandRegistry.register({
    id: 'forge.settings.open',
    title: 'Open Settings',
    category: 'Edit',
    keybinding: 'ctrl+,',
    globalWhenEditorFocused: true,
    execute: () => {
      activeDialog.open('settings');
      return null;
    },
  });

  commandRegistry.register({
    id: 'forge.settings.openKeybindings',
    title: 'Open Keybindings',
    category: 'Edit',
    keybinding: 'ctrl+k',
    globalWhenEditorFocused: true,
    execute: () => {
      activeDialog.open('keybindings');
      return null;
    },
  });

  commandRegistry.register({
    id: 'forge.settings.resetAll',
    title: 'Reset All Settings',
    category: 'Edit',
    execute: () => {
      appSettings.resetAll();
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
  registerMenuContribution({ menu: 'edit', group: 'settings', order: 10, commandId: 'forge.settings.open', label: 'Settings' });
  registerMenuContribution({ menu: 'edit', group: 'settings', order: 11, commandId: 'forge.settings.openKeybindings', label: 'Keybindings' });

  registerMenuContribution({ menu: 'view', group: 'fullscreen', order: 1, commandId: 'forge.window.toggleFullscreen', label: 'Toggle Fullscreen' });
  registerMenuContribution({ menu: 'view', group: 'theme', order: 2, commandId: 'forge.theme.toggle', label: 'Toggle Theme' });

  // 7. Install keyboard sovereignty
  const removeKeybindings = installKeybindingManager();

  // 8. File system + buffer map
  const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
  const fileSystem: ForgeFileSystem = isTauri ? new TauriFileSystem() : new MemoryFileSystem();

  // When in browser dev mode, seed the memory FS with mock .urd.md files
  if (!isTauri && fileSystem instanceof MemoryFileSystem) {
    fileSystem.seed({
      '/mock/urd-project/main.urd.md': '# World: Demo\n\n## Entity: Player\n- name: "Hero"\n',
      '/mock/urd-project/locations.urd.md': '## Location: Tavern\n- description: "A cosy tavern"\n',
    });
  }

  // On project open: scan for .urd.md files and populate buffer map
  const unsubProjectOpen = bus.subscribe('project.opened', async (payload) => {
    const { path } = payload as { path: string };
    bufferMap.clear();

    try {
      // Ensure .forge/ directory exists
      const forgePath = `${path}/.forge`;
      const forgeExists = await fileSystem.exists(forgePath);
      if (!forgeExists) {
        await fileSystem.mkdir(forgePath);
      }

      // Scan for .urd.md files
      const entries = await fileSystem.listDirectory(path);
      for (const entry of entries) {
        if (entry.isFile && entry.name.endsWith('.urd.md')) {
          const content = await fileSystem.readFile(entry.path);
          bufferMap.load(entry.path, content);
        }
      }
    } catch (err) {
      console.error('Failed to populate buffers on project open:', err);
    }
  });

  // On project close: clear buffer map and projections, stop pipeline
  const unsubProjectClose = bus.subscribe('project.closed', () => {
    recompilePipeline.stop();
    bufferMap.clear();
    projectionRegistry.clear();
  });

  // 8b. Register projections
  projectionRegistry.register(entityTableProjection);
  projectionRegistry.register(locationGraphProjection);
  projectionRegistry.register(diagnosticsByFileProjection);

  // 8c. Recompile pipeline
  const compilerService: CompilerService = isTauri
    ? new TauriCompiler()
    : new MockCompilerService();
  const compilerCache = new CompilerOutputCache();
  const recompilePipeline = new RecompilePipeline(
    bufferMap,
    compilerService,
    compilerCache,
    projectionRegistry,
    appSettings.get('recompileDebounceMs'),
  );

  // Start pipeline and trigger initial compile when project opens
  const unsubProjectOpenCompile = bus.subscribe('project.opened', () => {
    // Start watching for buffer changes
    recompilePipeline.start();
    // Trigger initial compile (after buffers are populated — use a microtask)
    queueMicrotask(() => {
      recompilePipeline.compileNow();
    });
  });

  // 9. Focus tracking — click on a zone viewport to focus it
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
    recompilePipeline.stop();
    unsubProjectOpen();
    unsubProjectClose();
    unsubProjectOpenCompile();
    document.removeEventListener('pointerdown', onZoneFocus);
    document.removeEventListener('pointerdown', onPointerDown);
    document.removeEventListener('pointerup', onPointerUp);
  };
}
