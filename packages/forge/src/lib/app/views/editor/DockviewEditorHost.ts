/**
 * DockviewEditorHost — IContentRenderer and ITabRenderer implementations
 * for hosting Svelte 5 EditorPanelContent components inside dockview panels.
 *
 * Uses Svelte 5's mount()/unmount() API to create and destroy component
 * instances inside dockview's container elements.
 */

import { mount, unmount } from 'svelte';
import type {
  IContentRenderer,
  ITabRenderer,
  GroupPanelPartInitParameters,
  TabPartInitParameters,
  Parameters,
} from 'dockview-core';
import EditorPanelContent from './EditorPanelContent.svelte';
import { bufferMap } from '$lib/app/compiler/BufferMap';

export interface EditorPanelParams extends Parameters {
  path: string;
}

/** Callback type for cross-file navigation (go-to-definition, etc.). */
export type NavigateCallback = (file: string, line: number, col?: number) => void;

/** Registry mapping panel IDs to their content renderers for external access. */
const rendererRegistry = new Map<string, EditorPanelRenderer>();

/** Get the EditorPanelRenderer for a given panel ID. */
export function getRenderer(panelId: string): EditorPanelRenderer | undefined {
  return rendererRegistry.get(panelId);
}

/**
 * Content renderer for editor panels. Each instance mounts an
 * EditorPanelContent Svelte component with its own CodeMirror editor.
 *
 * The `onNavigate` callback is injected at construction time (not through
 * dockview params) so it survives serialisation/deserialisation.
 */
export class EditorPanelRenderer implements IContentRenderer {
  readonly element: HTMLElement;
  private component: Record<string, unknown> | null = null;
  private panelId: string = '';

  constructor(private readonly onNavigate: NavigateCallback) {
    this.element = document.createElement('div');
    this.element.className = 'forge-dockview-panel-content';
    this.element.style.width = '100%';
    this.element.style.height = '100%';
    this.element.style.overflow = 'hidden';
  }

  init(parameters: GroupPanelPartInitParameters): void {
    const params = parameters.params as EditorPanelParams;
    this.panelId = parameters.api.id;

    this.component = mount(EditorPanelContent, {
      target: this.element,
      props: {
        filePath: params.path,
        onNavigate: this.onNavigate,
      },
    });

    rendererRegistry.set(this.panelId, this);
  }

  /** Scroll the CodeMirror instance inside this panel to a line. */
  scrollToLine(line: number, col?: number): void {
    if (this.component) {
      (this.component as unknown as { scrollToLine: (line: number, col?: number) => void }).scrollToLine(line, col);
    }
  }

  /** Focus the CodeMirror instance inside this panel. */
  focusEditor(): void {
    if (this.component) {
      (this.component as unknown as { focus: () => void }).focus();
    }
  }

  dispose(): void {
    rendererRegistry.delete(this.panelId);
    if (this.component) {
      unmount(this.component);
      this.component = null;
    }
  }
}

/**
 * Custom tab renderer matching forge design tokens.
 * Shows filename, dirty indicator, and close button.
 */
export class EditorTabRenderer implements ITabRenderer {
  readonly element: HTMLElement;
  private nameEl: HTMLSpanElement;
  private dirtyEl: HTMLSpanElement;
  private closeEl: HTMLButtonElement;
  private path: string = '';
  private disposeListeners: (() => void)[] = [];
  private bufferUnsub: (() => void) | null = null;

  constructor() {
    this.element = document.createElement('div');
    this.element.className = 'forge-dv-tab';

    this.nameEl = document.createElement('span');
    this.nameEl.className = 'forge-dv-tab__name';

    this.dirtyEl = document.createElement('span');
    this.dirtyEl.className = 'forge-dv-tab__dirty';
    this.dirtyEl.textContent = '\u25CF'; // ●
    this.dirtyEl.title = 'Unsaved changes';
    this.dirtyEl.style.display = 'none';

    this.closeEl = document.createElement('button');
    this.closeEl.className = 'forge-dv-tab__close';
    this.closeEl.textContent = '\u00D7'; // ×
    this.closeEl.title = 'Close';
    this.closeEl.setAttribute('aria-label', 'Close tab');

    this.element.appendChild(this.nameEl);
    this.element.appendChild(this.dirtyEl);
    this.element.appendChild(this.closeEl);
  }

  init(parameters: TabPartInitParameters): void {
    const params = parameters.params as EditorPanelParams;
    this.path = params.path;

    const filename = this.path.split('/').pop() || this.path;
    this.nameEl.textContent = filename;
    this.closeEl.setAttribute('aria-label', `Close ${filename}`);

    // Update dirty indicator
    this.updateDirty();

    // Subscribe to buffer changes for dirty tracking
    this.bufferUnsub = bufferMap.subscribe(() => {
      this.updateDirty();
    });

    // Close button handler
    const onClose = (e: MouseEvent) => {
      e.stopPropagation();
      e.preventDefault();
      parameters.api.close();
    };
    this.closeEl.addEventListener('mousedown', onClose);
    this.disposeListeners.push(() => this.closeEl.removeEventListener('mousedown', onClose));
  }

  private updateDirty(): void {
    const dirty = bufferMap.isDirty(this.path);
    this.dirtyEl.style.display = dirty ? '' : 'none';
  }

  dispose(): void {
    for (const cleanup of this.disposeListeners) cleanup();
    this.disposeListeners = [];
    if (this.bufferUnsub) {
      this.bufferUnsub();
      this.bufferUnsub = null;
    }
  }
}
