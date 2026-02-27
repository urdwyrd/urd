/**
 * Application bootstrap — registers all framework views, commands, and menus.
 * Phases 1–3: placeholders, framework commands, code editor, source viewer.
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
import { definitionIndexProjection } from '$lib/app/projections/definition-index';
import { typeTableProjection } from '$lib/app/projections/type-table';
import { propertyTableProjection } from '$lib/app/projections/property-table';
import { locationTableProjection } from '$lib/app/projections/location-table';
import { sectionTableProjection } from '$lib/app/projections/section-table';
import { worldStatsProjection } from '$lib/app/projections/world-stats';
import { deadCodeProjection } from '$lib/app/projections/dead-code';
import { outlineProjection } from '$lib/app/projections/outline';
import { urdJsonProjection } from '$lib/app/projections/urd-json';
import { symbolTableProjection } from '$lib/app/projections/symbol-table';
import { factSetProjection } from '$lib/app/projections/fact-set';
import { propertyDependencyIndexProjection } from '$lib/app/projections/property-dependency-index';
import { rawUrdJsonProjection } from '$lib/app/projections/raw-urd-json';
import { createWriterTemplate, createEngineerTemplate } from '$lib/app/workspaces/templates';
import type { CompilerService } from '$lib/app/compiler/types';

/** Exported file system instance — available after bootstrap(). */
export let fileSystem: ForgeFileSystem;

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

  // 4b. Register Urd application views
  viewRegistry.register({
    id: 'urd.codeEditor',
    name: 'Code Editor',
    icon: '◆',
    category: 'Editor',
    component: () => import('$lib/app/views/editor/CodeEditorZone.svelte'),
    navigationStrategy: 'singleton-autocreate',
    requiresProject: true,
    stateVersion: 2,
    defaultState: { dockviewLayout: null },
    migrateState: (oldState: unknown, fromVersion: number) => {
      if (fromVersion === 1) {
        // v1 had { openTabs: TabState[], activeTab: string | null }
        // v2 uses { dockviewLayout: SerializedDockview | null }
        // We discard the old layout — tabs will be empty on first open.
        // The active file will be re-opened by the project.opened handler.
        return { dockviewLayout: null };
      }
      return oldState;
    },
  });

  viewRegistry.register({
    id: 'urd.sourceViewer',
    name: 'Source Viewer',
    icon: '▸',
    category: 'Editor',
    component: () => import('$lib/app/views/source-viewer/SourceViewerZone.svelte'),
    navigationStrategy: 'multi',
    requiresProject: true,
    stateVersion: 1,
    defaultState: { filePath: null, scrollTop: 0 },
  });

  // 4c. Spreadsheet views
  viewRegistry.register({
    id: 'urd.entitySpreadsheet',
    name: 'Entity Spreadsheet',
    icon: '▸',
    category: 'Spreadsheets',
    component: () => import('$lib/app/views/spreadsheets/EntitySpreadsheet.svelte'),
    requiresProject: true,
    stateVersion: 1,
    defaultState: { sort: null, filterText: '' },
  });

  viewRegistry.register({
    id: 'urd.typeSpreadsheet',
    name: 'Type Spreadsheet',
    icon: '▸',
    category: 'Spreadsheets',
    component: () => import('$lib/app/views/spreadsheets/TypeSpreadsheet.svelte'),
    requiresProject: true,
    stateVersion: 1,
    defaultState: { sort: null, filterText: '' },
  });

  viewRegistry.register({
    id: 'urd.propertySpreadsheet',
    name: 'Property Spreadsheet',
    icon: '▸',
    category: 'Spreadsheets',
    component: () => import('$lib/app/views/spreadsheets/PropertySpreadsheet.svelte'),
    requiresProject: true,
    stateVersion: 1,
    defaultState: { sort: null, filterText: '' },
  });

  viewRegistry.register({
    id: 'urd.locationSpreadsheet',
    name: 'Location Spreadsheet',
    icon: '▸',
    category: 'Spreadsheets',
    component: () => import('$lib/app/views/spreadsheets/LocationSpreadsheet.svelte'),
    requiresProject: true,
    stateVersion: 1,
    defaultState: { sort: null, filterText: '' },
  });

  viewRegistry.register({
    id: 'urd.sectionSpreadsheet',
    name: 'Section Spreadsheet',
    icon: '▸',
    category: 'Spreadsheets',
    component: () => import('$lib/app/views/spreadsheets/SectionSpreadsheet.svelte'),
    requiresProject: true,
    stateVersion: 1,
    defaultState: { sort: null, filterText: '' },
  });

  viewRegistry.register({
    id: 'urd.diagnosticSpreadsheet',
    name: 'Diagnostic Spreadsheet',
    icon: '▸',
    category: 'Spreadsheets',
    component: () => import('$lib/app/views/spreadsheets/DiagnosticSpreadsheet.svelte'),
    requiresProject: true,
    stateVersion: 1,
    defaultState: { sort: null, filterText: '' },
  });

  // 4e. Navigation views
  viewRegistry.register({
    id: 'urd.fileBrowser',
    name: 'File Browser',
    icon: '▸',
    category: 'Navigation',
    component: () => import('$lib/app/views/file-browser/FileBrowser.svelte'),
    navigationStrategy: 'multi',
    requiresProject: false,
    stateVersion: 1,
    defaultState: { expandedPaths: [] },
  });

  // 4g. Inspector views
  viewRegistry.register({
    id: 'urd.propertyInspector',
    name: 'Property Inspector',
    icon: '▸',
    category: 'Inspectors',
    component: () => import('$lib/app/views/inspectors/PropertyInspector.svelte'),
    requiresProject: true,
    stateVersion: 1,
    defaultState: null,
  });

  // 4h. Analysis views
  viewRegistry.register({
    id: 'urd.worldStatsDashboard',
    name: 'World Stats',
    icon: '▸',
    category: 'Analysis',
    component: () => import('$lib/app/views/analysis/WorldStatsDashboard.svelte'),
    requiresProject: true,
    stateVersion: 1,
    defaultState: null,
  });

  viewRegistry.register({
    id: 'urd.deadCodePanel',
    name: 'Dead Code',
    icon: '▸',
    category: 'Analysis',
    component: () => import('$lib/app/views/analysis/DeadCodePanel.svelte'),
    requiresProject: true,
    stateVersion: 1,
    defaultState: null,
  });

  // 4i. Search/Outline views
  viewRegistry.register({
    id: 'urd.outlinePanel',
    name: 'Outline',
    icon: '▸',
    category: 'Navigation',
    component: () => import('$lib/app/views/search/OutlinePanel.svelte'),
    requiresProject: true,
    stateVersion: 1,
    defaultState: null,
  });

  viewRegistry.register({
    id: 'urd.globalSymbolSearch',
    name: 'Symbol Search',
    icon: '▸',
    category: 'Navigation',
    component: () => import('$lib/app/views/search/GlobalSymbolSearch.svelte'),
    requiresProject: true,
    stateVersion: 1,
    defaultState: null,
  });

  // 4k. Settings/keybindings open as floating dialogs (not zone views)

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

  // 5b. Editor commands
  commandRegistry.register({
    id: 'forge.editor.openFile',
    title: 'Open File in Editor',
    category: 'Editor',
    execute: (...args: unknown[]) => {
      const opts = args[0] as { path: string; line?: number; col?: number } | undefined;
      if (opts?.path) {
        navigationBroker.navigate({
          targetViewId: 'urd.codeEditor',
          params: { path: opts.path, line: opts.line, col: opts.col },
        });
      }
      return null;
    },
  });

  commandRegistry.register({
    id: 'forge.editor.closeTab',
    title: 'Close Tab',
    category: 'Editor',
    keybinding: 'ctrl+w',
    execute: () => {
      // Dispatched to the active editor zone via bus
      if (bus.hasChannel('editor.closeTab')) {
        bus.publish('editor.closeTab', {});
      }
      return null;
    },
  });

  commandRegistry.register({
    id: 'forge.editor.nextTab',
    title: 'Next Tab',
    category: 'Editor',
    keybinding: 'ctrl+tab',
    execute: () => {
      if (bus.hasChannel('editor.nextTab')) {
        bus.publish('editor.nextTab', {});
      }
      return null;
    },
  });

  commandRegistry.register({
    id: 'forge.editor.prevTab',
    title: 'Previous Tab',
    category: 'Editor',
    keybinding: 'ctrl+shift+tab',
    execute: () => {
      if (bus.hasChannel('editor.prevTab')) {
        bus.publish('editor.prevTab', {});
      }
      return null;
    },
  });

  commandRegistry.register({
    id: 'forge.editor.save',
    title: 'Save File',
    category: 'Editor',
    keybinding: 'ctrl+s',
    execute: () => {
      if (bus.hasChannel('editor.save')) {
        bus.publish('editor.save', {});
      }
      return null;
    },
  });

  commandRegistry.register({
    id: 'forge.fileBrowser.focus',
    title: 'Focus File Browser',
    category: 'Navigation',
    keybinding: 'ctrl+shift+e',
    globalWhenEditorFocused: true,
    execute: () => {
      navigationBroker.navigate({ targetViewId: 'urd.fileBrowser' });
      return null;
    },
  });

  commandRegistry.register({
    id: 'forge.search.globalSymbol',
    title: 'Go to Symbol',
    category: 'Navigation',
    keybinding: 'ctrl+t',
    globalWhenEditorFocused: true,
    execute: () => {
      navigationBroker.navigate({ targetViewId: 'urd.globalSymbolSearch' });
      return null;
    },
  });

  // 5c. Zone operation commands (used by context menus)
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

  // 5d. Workspace template commands
  workspaceManager.registerTemplate('Writer', createWriterTemplate);
  workspaceManager.registerTemplate('Engineer', createEngineerTemplate);

  commandRegistry.register({
    id: 'forge.workspace.newWriter',
    title: 'New Writer Workspace',
    category: 'Workspace',
    execute: () => {
      workspaceManager.createFromTemplate('Writer');
      return null;
    },
  });

  commandRegistry.register({
    id: 'forge.workspace.newEngineer',
    title: 'New Engineer Workspace',
    category: 'Workspace',
    execute: () => {
      workspaceManager.createFromTemplate('Engineer');
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
  fileSystem = isTauri ? new TauriFileSystem() : new MemoryFileSystem();

  // When in browser dev mode, seed the memory FS with mock .urd.md files
  if (!isTauri && fileSystem instanceof MemoryFileSystem) {
    fileSystem.seed({
      '/mock/urd-project/main.urd.md': '# World: Demo\n\n## Entity: Player\n- name: "Hero"\n',
      '/mock/urd-project/locations.urd.md': '## Location: Tavern\n- description: "A cosy tavern"\n',
    });
  }

  // On project open: start pipeline, scan for .urd.md files, then compile.
  // Pipeline starts first so buffer.load() calls during scanning schedule
  // debounced recompiles. After scanning completes we force an immediate compile.
  const unsubProjectOpen = bus.subscribe('project.opened', async (payload) => {
    const { path } = payload as { path: string };
    bufferMap.clear();
    recompilePipeline.start();

    try {
      // Ensure .forge/ directory exists
      const forgePath = `${path}/.forge`;
      const forgeExists = await fileSystem.exists(forgePath);
      if (!forgeExists) {
        await fileSystem.mkdir(forgePath);
      }

      // Recursively scan for .urd.md files
      async function scanDir(dirPath: string): Promise<void> {
        const entries = await fileSystem.listDirectory(dirPath);
        for (const entry of entries) {
          if (entry.isDirectory && !entry.name.startsWith('.')) {
            await scanDir(entry.path);
          } else if (entry.isFile && entry.name.endsWith('.urd.md')) {
            const content = await fileSystem.readFile(entry.path);
            bufferMap.load(entry.path, content);
          }
        }
      }
      await scanDir(path);
    } catch (err) {
      console.error('Failed to populate buffers on project open:', err);
    }

    // Determine the first .urd.md file for both compilation and editor auto-open.
    const paths = bufferMap.paths();
    const firstUrdFile = paths.find((p) => p.endsWith('.urd.md'));

    // Publish the active file BEFORE compiling so the pipeline uses the correct
    // entry point. Without this, activeFile is null and the bridge falls back to
    // the alphabetically first file in the buffer map.
    if (firstUrdFile) {
      bus.publish('editor.activeFile', { path: firstUrdFile });
    }

    // All buffers loaded — compile immediately (cancels any pending debounce)
    recompilePipeline.compileNow();

    // Auto-open the Code Editor zone with the first .urd.md file
    if (firstUrdFile) {
      navigationBroker.navigate({
        targetViewId: 'urd.codeEditor',
        params: { path: firstUrdFile },
      });
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
  projectionRegistry.register(definitionIndexProjection);
  projectionRegistry.register(typeTableProjection);
  projectionRegistry.register(propertyTableProjection);
  projectionRegistry.register(locationTableProjection);
  projectionRegistry.register(sectionTableProjection);
  projectionRegistry.register(worldStatsProjection);
  projectionRegistry.register(deadCodeProjection);
  projectionRegistry.register(outlineProjection);
  projectionRegistry.register(urdJsonProjection);
  projectionRegistry.register(symbolTableProjection);
  projectionRegistry.register(factSetProjection);
  projectionRegistry.register(propertyDependencyIndexProjection);
  projectionRegistry.register(rawUrdJsonProjection);

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

  // (Pipeline start + initial compile + auto-open are handled in the
  //  project.opened handler above, after buffer population completes.)

  // Auto-reopen last project on startup (or mock project in browser dev mode)
  if (!isTauri) {
    // Browser dev: open mock project immediately so the editor has data
    projectManager.openPath('/mock/urd-project');
  } else {
    const lastPath = appSettings.get('lastProjectPath');
    if (lastPath) {
      projectManager.openPath(lastPath);
    }
  }

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
    document.removeEventListener('pointerdown', onZoneFocus);
    document.removeEventListener('pointerdown', onPointerDown);
    document.removeEventListener('pointerup', onPointerUp);
  };
}
