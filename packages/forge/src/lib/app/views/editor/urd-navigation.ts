/**
 * Go-to-definition and visual link affordance for the Forge editor.
 *
 * Adapted from the playground's goto-definition.ts. Instead of getState(),
 * accepts a resolveDefinition callback provided by CodeEditorZone.
 * For cross-file navigation, calls the onNavigate callback.
 */

import { ViewPlugin, Decoration, EditorView, type DecorationSet } from '@codemirror/view';
import { StateField, StateEffect } from '@codemirror/state';
import { identifyReference, type Reference } from './cursor-resolver';
import { byteColToCharCol } from './urd-diagnostics';
import { filesMatch, resolveCompilerPath } from './file-match';
import type { RichDefinitionEntry } from '$lib/app/compiler/types';

// --- Types ---

export type DefinitionEntry = RichDefinitionEntry;

export interface DefinitionResolver {
  /** Look up a definition entry by its key, returning the entry or null. */
  findByKey(key: string): DefinitionEntry | null;
  /** Find a definition entry matching a predicate. */
  find(predicate: (entry: DefinitionEntry) => boolean): DefinitionEntry | null;
  /** Resolve entity type name from entity ID. */
  getEntityTypeName?(entityId: string): string | null;
}

export interface NavigationCallback {
  /** Navigate to a position. If file differs from current, opens in a new tab. */
  (file: string, line: number, col?: number): void;
}

// --- Go-to-definition extension ---

/**
 * Creates the go-to-definition click handler extension.
 * Call this with callback providers that resolve from Forge's projection registry.
 */
export function urdGotoDefinition(
  getResolver: () => DefinitionResolver | null,
  onNavigate: NavigationCallback,
  getCurrentFile: () => string | null,
) {
  return ViewPlugin.fromClass(
    class {
      handleClick: (e: MouseEvent) => void;

      constructor(readonly view: EditorView) {
        this.handleClick = (e: MouseEvent) => {
          if (!(e.ctrlKey || e.metaKey)) return;
          const pos = view.posAtCoords({ x: e.clientX, y: e.clientY });
          if (pos == null) return;

          const line = view.state.doc.lineAt(pos);
          const col = pos - line.from;
          const ref = identifyReference(line.text, col);
          if (!ref) return;

          const resolver = getResolver();
          if (!resolver) return;

          const target = resolveTarget(ref, resolver);
          if (!target) return;

          e.preventDefault();

          const currentFile = getCurrentFile();
          if (target.file && currentFile && !filesMatch(currentFile, target.file)) {
            // Cross-file navigation
            onNavigate(target.file, target.line, target.col);
          } else {
            // Same-file navigation
            const targetLine = view.state.doc.line(
              Math.min(target.line, view.state.doc.lines)
            );
            const charCol = target.col ? Math.max(0, target.col - 1) : 0;
            const targetPos = targetLine.from + charCol;

            view.dispatch({
              selection: { anchor: targetPos },
              scrollIntoView: true,
            });
            view.focus();
          }
        };

        view.dom.addEventListener('click', this.handleClick);
      }

      destroy() {
        this.view.dom.removeEventListener('click', this.handleClick);
      }
    },
  );
}

// --- Visual affordance (underline on Ctrl/Cmd hover) ---

const setLinkDeco = StateEffect.define<DecorationSet>();

const linkField = StateField.define<DecorationSet>({
  create: () => Decoration.none,
  update(value, tr) {
    for (const e of tr.effects) {
      if (e.is(setLinkDeco)) return e.value;
    }
    return value;
  },
  provide: (f) => EditorView.decorations.from(f),
});

/**
 * Creates the definition link visual affordance extension.
 */
export function urdDefinitionLink(
  getResolver: () => DefinitionResolver | null,
) {
  const plugin = ViewPlugin.fromClass(
    class {
      private modDown = false;

      constructor(readonly view: EditorView) {
        this.onKeyDown = this.onKeyDown.bind(this);
        this.onKeyUp = this.onKeyUp.bind(this);
        this.onMouseMove = this.onMouseMove.bind(this);
        this.onMouseLeave = this.onMouseLeave.bind(this);

        view.dom.addEventListener('keydown', this.onKeyDown);
        view.dom.addEventListener('keyup', this.onKeyUp);
        view.dom.addEventListener('mousemove', this.onMouseMove);
        view.dom.addEventListener('mouseleave', this.onMouseLeave);
      }

      onKeyDown(e: KeyboardEvent) {
        if (e.key === 'Control' || e.key === 'Meta') {
          this.modDown = true;
        }
      }

      onKeyUp(e: KeyboardEvent) {
        if (e.key === 'Control' || e.key === 'Meta') {
          this.modDown = false;
          this.clearDeco();
        }
      }

      onMouseMove(e: MouseEvent) {
        if (!this.modDown) return;
        const pos = this.view.posAtCoords({ x: e.clientX, y: e.clientY });
        if (pos == null) { this.clearDeco(); return; }

        const line = this.view.state.doc.lineAt(pos);
        const col = pos - line.from;
        const ref = identifyReference(line.text, col);
        const resolver = getResolver();

        if (!ref || !resolver || !resolveTarget(ref, resolver)) {
          this.clearDeco();
          return;
        }

        const span = getTokenSpan(line.text, col);
        if (!span) { this.clearDeco(); return; }

        const from = line.from + span.from;
        const to = line.from + span.to;
        const deco = Decoration.set([
          Decoration.mark({ class: 'cm-definition-link' }).range(from, to),
        ]);
        this.view.dispatch({ effects: setLinkDeco.of(deco) });
      }

      onMouseLeave() {
        this.modDown = false;
        this.clearDeco();
      }

      clearDeco() {
        this.view.dispatch({ effects: setLinkDeco.of(Decoration.none) });
      }

      destroy() {
        this.view.dom.removeEventListener('keydown', this.onKeyDown);
        this.view.dom.removeEventListener('keyup', this.onKeyUp);
        this.view.dom.removeEventListener('mousemove', this.onMouseMove);
        this.view.dom.removeEventListener('mouseleave', this.onMouseLeave);
      }
    },
  );

  return [linkField, plugin];
}

// --- Resolution helpers ---

interface ResolvedTarget {
  file: string;
  line: number;
  col?: number;
}

function resolveTarget(ref: Reference, resolver: DefinitionResolver): ResolvedTarget | null {
  let key: string | null = null;

  switch (ref.kind) {
    case 'entity':
      key = `entity:@${ref.id}`;
      break;
    case 'entity-property': {
      const typeName = resolver.getEntityTypeName?.(ref.entityId);
      if (!typeName) return null;
      key = `prop:${typeName}.${ref.property}`;
      break;
    }
    case 'type-property':
      key = `prop:${ref.typeName}.${ref.property}`;
      break;
    case 'section-jump':
    case 'section-label': {
      const entry = resolver.find(
        (d) => d.definition.kind === 'section' && d.definition.local_name === ref.name,
      );
      if (!entry) return null;
      return spanToTarget(entry.span);
    }
    case 'location-heading': {
      const entry = resolver.find(
        (d) => d.definition.kind === 'location' && d.definition.display_name === ref.name,
      );
      if (!entry) return null;
      return spanToTarget(entry.span);
    }
  }

  if (!key) return null;
  const entry = resolver.findByKey(key);
  if (!entry) return null;
  return spanToTarget(entry.span);
}

function spanToTarget(span: { file: string; start_line: number; start_col: number }): ResolvedTarget {
  return {
    file: span.file,
    line: span.start_line,
    col: span.start_col,
  };
}

/** Find the word-like token boundaries around a column position. */
function getTokenSpan(lineText: string, col: number): { from: number; to: number } | null {
  if (col < 0 || col >= lineText.length) return null;

  let from = col;
  let to = col;

  while (to < lineText.length && isTokenChar(lineText[to])) to++;
  while (from > 0 && isTokenChar(lineText[from - 1])) from--;

  // Include @ prefix
  if (from > 0 && lineText[from - 1] === '@') from--;

  if (from >= to) return null;
  return { from, to };
}

function isTokenChar(ch: string): boolean {
  return (ch >= 'a' && ch <= 'z') ||
    (ch >= 'A' && ch <= 'Z') ||
    (ch >= '0' && ch <= '9') ||
    ch === '_' || ch === '.';
}

// Re-export byteColToCharCol for use in other modules
export { byteColToCharCol };
