/**
 * Gutter annotations — diagnostic and definition markers in the editor gutter.
 *
 * Red dot for error lines, amber for warnings, entity marker for definitions.
 * Uses CodeMirror gutter() facet + GutterMarker.
 */

import { gutter, GutterMarker } from '@codemirror/view';
import { RangeSet, RangeSetBuilder, type Extension } from '@codemirror/state';
import type { FileDiagnostics } from '$lib/app/projections/diagnostics-by-file';
import type { DefinitionIndex } from '$lib/app/projections/definition-index';
import { filesMatch } from './file-match';

export interface GutterDataProvider {
  getDiagnosticsByFile(): FileDiagnostics[] | null;
  getDefinitionIndex(): DefinitionIndex | null;
  getCurrentFile(): string | null;
}

class ErrorMarker extends GutterMarker {
  toDOM(): Node {
    const el = document.createElement('span');
    el.className = 'forge-gutter-marker forge-gutter-marker--error';
    el.textContent = '\u25CF';
    el.title = 'Error';
    return el;
  }
}

class WarningMarker extends GutterMarker {
  toDOM(): Node {
    const el = document.createElement('span');
    el.className = 'forge-gutter-marker forge-gutter-marker--warning';
    el.textContent = '\u25CF';
    el.title = 'Warning';
    return el;
  }
}

class DefinitionMarker extends GutterMarker {
  constructor(private kind: string) {
    super();
  }

  toDOM(): Node {
    const el = document.createElement('span');
    el.className = `forge-gutter-marker forge-gutter-marker--${this.kind}`;
    el.textContent = '\u25C6';
    el.title = this.kind;
    return el;
  }
}

const errorMarker = new ErrorMarker();
const warningMarker = new WarningMarker();

/**
 * Creates the gutter extension with diagnostic and definition markers.
 */
export function urdGutter(getProvider: () => GutterDataProvider | null): Extension {
  return gutter({
    class: 'forge-urd-gutter',
    markers: (view) => {
      try {
        return buildGutterMarkers(view, getProvider);
      } catch {
        return RangeSet.empty;
      }
    },
    initialSpacer: () => errorMarker,
  });
}

function buildGutterMarkers(
  view: { state: { doc: { lines: number; line: (n: number) => { from: number } } } },
  getProvider: () => GutterDataProvider | null,
) {
      const provider = getProvider();
      if (!provider) return RangeSet.empty;

      const currentFile = provider.getCurrentFile();
      if (!currentFile) return RangeSet.empty;

      const builder = new RangeSetBuilder<GutterMarker>();
      const doc = view.state.doc;
      const markerMap = new Map<number, GutterMarker>();

      // Add diagnostic markers (errors override warnings)
      const diagsByFile = provider.getDiagnosticsByFile();
      if (diagsByFile) {
        const fileDiag = diagsByFile.find((fd) => filesMatch(currentFile, fd.file));
        if (fileDiag) {
          for (const diag of fileDiag.diagnostics) {
            if (!diag.span) continue;
            const lineNum = diag.span.startLine;
            if (lineNum < 1 || lineNum > doc.lines) continue;

            const existing = markerMap.get(lineNum);
            if (diag.severity === 'error') {
              markerMap.set(lineNum, errorMarker);
            } else if (diag.severity === 'warning' && !existing) {
              markerMap.set(lineNum, warningMarker);
            }
          }
        }
      }

      // Add definition markers (only if no diagnostic on that line)
      const defIndex = provider.getDefinitionIndex();
      if (defIndex) {
        for (const def of defIndex) {
          if (!filesMatch(currentFile, def.span.file)) continue;
          const lineNum = def.span.start_line;
          if (lineNum < 1 || lineNum > doc.lines) continue;
          if (!markerMap.has(lineNum)) {
            markerMap.set(lineNum, new DefinitionMarker(def.definition.kind));
          }
        }
      }

      // Build in line order
      const sortedLines = [...markerMap.keys()].sort((a, b) => a - b);
      for (const lineNum of sortedLines) {
        const lineObj = doc.line(lineNum);
        builder.add(lineObj.from, lineObj.from, markerMap.get(lineNum)!);
      }

      return builder.finish();
}
