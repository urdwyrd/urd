/**
 * Diagnostics integration for the Forge editor.
 *
 * Adapted from the playground's lint-source.ts. Instead of calling the WASM
 * compiler, this module uses the imperative `setDiagnostics()` API to push
 * diagnostics from the RecompilePipeline/ProjectionRegistry into CodeMirror.
 */

import { setDiagnostics, type Diagnostic as CMLintDiag } from '@codemirror/lint';
import type { EditorView } from '@codemirror/view';
import type { Diagnostic } from '$lib/app/compiler/types';

/**
 * Convert a Rust byte-offset column to a JavaScript character column.
 *
 * The Urd compiler reports columns as byte offsets (UTF-8). JS strings use
 * UTF-16 code units. For ASCII, they're identical, but multi-byte characters
 * (e.g., emoji, accented characters) cause divergence.
 */
export function byteColToCharCol(lineText: string, byteCol: number): number {
  const encoder = new TextEncoder();
  let byteCount = 0;
  for (let i = 0; i < lineText.length; i++) {
    const char = lineText[i];
    const bytes = encoder.encode(char);
    byteCount += bytes.length;
    if (byteCount >= byteCol) {
      return i + 1; // 1-based column
    }
  }
  return lineText.length + 1;
}

/**
 * Map Forge diagnostics to CodeMirror diagnostics and push them into the view.
 *
 * Call this after each `compiler.completed` signal with the diagnostics for
 * the currently active file.
 */
export function setEditorDiagnostics(
  view: EditorView,
  fileDiagnostics: Diagnostic[],
): void {
  const doc = view.state.doc;
  const cmDiagnostics: CMLintDiag[] = [];

  for (const d of fileDiagnostics) {
    if (!d.span) continue;

    const startLine = doc.line(Math.min(d.span.startLine, doc.lines));
    const endLine = doc.line(Math.min(d.span.endLine, doc.lines));
    const startCharCol = byteColToCharCol(startLine.text, d.span.startCol);
    const endCharCol = byteColToCharCol(endLine.text, d.span.endCol);
    const from = startLine.from + Math.max(0, startCharCol - 1);
    const to = endLine.from + Math.max(0, endCharCol - 1);

    cmDiagnostics.push({
      from: Math.max(0, from),
      to: Math.max(from + 1, to),
      severity: mapSeverity(d.severity),
      message: `[${d.code}] ${d.message}`,
    });
  }

  view.dispatch(setDiagnostics(view.state, cmDiagnostics));
}

/** Clear all diagnostics from the view. */
export function clearEditorDiagnostics(view: EditorView): void {
  view.dispatch(setDiagnostics(view.state, []));
}

function mapSeverity(s: string): 'error' | 'warning' | 'info' {
  if (s === 'error') return 'error';
  if (s === 'warning') return 'warning';
  return 'info';
}
