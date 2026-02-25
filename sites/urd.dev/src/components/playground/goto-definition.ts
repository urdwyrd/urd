import { ViewPlugin, Decoration, EditorView, type DecorationSet } from '@codemirror/view';
import { StateField, StateEffect } from '@codemirror/state';
import { identifyReference, type Reference } from './cursor-resolver';
import { getState } from './playground-state';
import { byteColToCharCol } from './compiler-bridge';

/**
 * Go-to-definition: Ctrl/Cmd+click jumps to the declaration of the
 * entity, property, section, or location under the cursor.
 */
export function urdGotoDefinition() {
  return gotoPlugin;
}

/**
 * Visual affordance: underline + pointer cursor when Ctrl/Cmd is held
 * over a resolvable reference.
 */
export function urdDefinitionLink() {
  return [linkField, linkPlugin];
}

// --- Go-to-definition click handler ---

const gotoPlugin = ViewPlugin.fromClass(
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

        const targetPos = resolveDefinitionPos(ref, view);
        if (targetPos == null) return;

        e.preventDefault();
        view.dispatch({
          selection: { anchor: targetPos },
          scrollIntoView: true,
        });
        view.focus();
      };

      view.dom.addEventListener('click', this.handleClick);
    }

    destroy() {
      this.view.dom.removeEventListener('click', this.handleClick);
    }
  },
);

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

const linkPlugin = ViewPlugin.fromClass(
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

      if (!ref || resolveDefinitionPos(ref, this.view) == null) {
        this.clearDeco();
        return;
      }

      // Find the token span to underline
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

// --- Resolution helpers ---

function resolveDefinitionPos(ref: Reference, view: EditorView): number | null {
  const state = getState();
  if (!state.definitionIndex) return null;

  let key: string | null = null;

  switch (ref.kind) {
    case 'entity':
      key = `entity:@${ref.id}`;
      break;
    case 'entity-property': {
      const typeName = state.parsedWorld?.entities?.[ref.entityId]?.type;
      if (!typeName) return null;
      key = `prop:${typeName}.${ref.property}`;
      break;
    }
    case 'type-property':
      key = `prop:${ref.typeName}.${ref.property}`;
      break;
    case 'section-jump':
    case 'section-label': {
      const entry = state.definitionIndex.find(
        (d) => d.definition.kind === 'section' && d.definition.local_name === ref.name,
      );
      if (!entry) return null;
      return spanToPos(entry.span, view);
    }
    case 'location-heading': {
      const entry = state.definitionIndex.find(
        (d) => d.definition.kind === 'location' && d.definition.display_name === ref.name,
      );
      if (!entry) return null;
      return spanToPos(entry.span, view);
    }
  }

  if (!key) return null;
  const entry = state.definitionIndex.find((d) => d.key === key);
  if (!entry) return null;
  return spanToPos(entry.span, view);
}

function spanToPos(
  span: { start_line: number; start_col: number },
  view: EditorView,
): number {
  const line = view.state.doc.line(Math.min(span.start_line, view.state.doc.lines));
  const charCol = byteColToCharCol(line.text, span.start_col);
  return line.from + Math.max(0, charCol - 1);
}

/** Find the word-like token boundaries around a column position. */
function getTokenSpan(lineText: string, col: number): { from: number; to: number } | null {
  if (col < 0 || col >= lineText.length) return null;

  // Expand from col to include word chars and @ prefix
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
