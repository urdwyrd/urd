/**
 * Code Lens extension — shows "N references" above entity/location definitions.
 *
 * Uses a StateField for block decorations (CodeMirror requires block widgets
 * to come from StateFields, not ViewPlugins). A ViewPlugin rebuilds the
 * decoration set and pushes it into the field via a deferred StateEffect
 * dispatch (requestAnimationFrame to avoid re-entrant dispatch).
 */

import { ViewPlugin, Decoration, EditorView, WidgetType, type DecorationSet } from '@codemirror/view';
import { StateField, StateEffect, RangeSetBuilder, type Extension } from '@codemirror/state';
import type { EntityRow } from '$lib/app/projections/entity-table';
import type { PropertyRow } from '$lib/app/projections/property-table';
import type { DefinitionIndex } from '$lib/app/projections/definition-index';
import { filesMatch } from './file-match';

export interface CodeLensDataProvider {
  getEntityTable(): EntityRow[] | null;
  getPropertyTable(): PropertyRow[] | null;
  getDefinitionIndex(): DefinitionIndex | null;
  getCurrentFile(): string | null;
}

class CodeLensWidget extends WidgetType {
  constructor(private text: string) {
    super();
  }

  toDOM(): HTMLElement {
    const span = document.createElement('div');
    span.className = 'forge-code-lens';
    span.textContent = this.text;
    return span;
  }

  eq(other: CodeLensWidget): boolean {
    return this.text === other.text;
  }

  get estimatedHeight(): number {
    return 16;
  }

  ignoreEvent(): boolean {
    return true;
  }
}

// Effect to push new block decorations into the state field
const setCodeLensDeco = StateEffect.define<DecorationSet>();

// StateField that holds the block decorations — required by CodeMirror for block widgets
const codeLensField = StateField.define<DecorationSet>({
  create: () => Decoration.none,
  update(value, tr) {
    for (const e of tr.effects) {
      if (e.is(setCodeLensDeco)) return e.value;
    }
    // Map existing decorations through document changes so positions stay valid
    if (tr.docChanged) return value.map(tr.changes);
    return value;
  },
  provide: (f) => EditorView.decorations.from(f),
});

/**
 * Creates the code lens extension (StateField + ViewPlugin pair).
 * Provider getter is called each time decorations are rebuilt.
 */
export function urdCodeLens(getProvider: () => CodeLensDataProvider | null): Extension {
  const plugin = ViewPlugin.fromClass(
    class {
      private pendingRaf = 0;
      private lastDecoJson = '';

      constructor(readonly view: EditorView) {
        this.scheduleUpdate();
      }

      update(): void {
        this.scheduleUpdate();
      }

      destroy(): void {
        if (this.pendingRaf) cancelAnimationFrame(this.pendingRaf);
      }

      private scheduleUpdate(): void {
        if (this.pendingRaf) cancelAnimationFrame(this.pendingRaf);
        this.pendingRaf = requestAnimationFrame(() => {
          this.pendingRaf = 0;
          this.pushDecorations();
        });
      }

      private pushDecorations(): void {
        const decos = this.buildDecorations();
        // Avoid redundant dispatches by comparing a lightweight signature
        const sig = this.decoSignature(decos);
        if (sig === this.lastDecoJson) return;
        this.lastDecoJson = sig;
        this.view.dispatch({ effects: setCodeLensDeco.of(decos) });
      }

      /** Cheap signature to detect whether decorations changed. */
      private decoSignature(decos: DecorationSet): string {
        if (decos === Decoration.none) return '';
        // Count widgets — good enough to detect data changes
        let count = 0;
        const iter = decos.iter();
        while (iter.value) { count++; iter.next(); }
        return `${count}`;
      }

      buildDecorations(): DecorationSet {
        try {
          return this.buildDecorationsInner();
        } catch {
          return Decoration.none;
        }
      }

      buildDecorationsInner(): DecorationSet {
        const provider = getProvider();
        if (!provider) return Decoration.none;

        const currentFile = provider.getCurrentFile();
        if (!currentFile) return Decoration.none;

        const defIndex = provider.getDefinitionIndex();
        if (!defIndex) return Decoration.none;

        const entities = provider.getEntityTable() ?? [];
        const properties = provider.getPropertyTable() ?? [];

        // Collect reference counts per symbol
        const refCounts = new Map<string, number>();

        // Count properties referencing each entity
        for (const entity of entities) {
          const refs = properties.filter(
            (p) => p.object === entity.id || p.object === entity.name
          ).length;
          if (refs > 0) {
            refCounts.set(entity.id, refs);
          }
        }

        // Build decorations for definitions in the current file.
        // Compiler spans use short filenames — match by suffix via filesMatch.
        const builder = new RangeSetBuilder<Decoration>();
        const fileDefs = defIndex
          .filter((d) => filesMatch(currentFile, d.span.file))
          .sort((a, b) => a.span.start_line - b.span.start_line);

        const doc = this.view.state.doc;

        for (const def of fileDefs) {
          if (def.span.start_line < 1 || def.span.start_line > doc.lines) continue;

          const kind = def.definition.kind;
          if (kind !== 'entity' && kind !== 'location') continue;

          // Find reference count
          const id = def.key.split(':')[1];
          // Look up entity id from def key (entity:@Name -> ent_name pattern)
          const entityRow = entities.find(
            (e) => e.name === id?.replace('@', '') || e.name === def.definition.display_name
          );
          const count = entityRow ? (refCounts.get(entityRow.id) ?? 0) : 0;

          // Only show code lens when there are actual references
          if (count === 0) continue;

          const lineObj = doc.line(def.span.start_line);
          const text = `${kind} \u00B7 ${count} reference${count === 1 ? '' : 's'}`;

          const widget = Decoration.widget({
            widget: new CodeLensWidget(text),
            side: -1,
            block: true,
          });

          builder.add(lineObj.from, lineObj.from, widget);
        }

        return builder.finish();
      }
    },
  );

  return [codeLensField, plugin];
}
