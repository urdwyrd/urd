import { linter, type Diagnostic as CMLintDiag } from '@codemirror/lint';
import type { Text } from '@codemirror/state';
import { compileSource, byteColToCharCol, type CompileResult } from './compiler-bridge';
import { updateState } from './playground-state';

/**
 * CodeMirror linter extension that runs the WASM compiler on every edit
 * (debounced at 300ms) and maps diagnostics to inline squiggly underlines.
 * Also updates the shared playground state for other extensions to consume.
 */
export function urdLinter() {
  return linter(
    (view) => {
      const source = view.state.doc.toString();
      const t0 = performance.now();
      const result = compileSource(source);
      const elapsed = performance.now() - t0;
      updateState(result, elapsed);
      return mapDiagnostics(result, view.state.doc);
    },
    { delay: 300 },
  );
}

function mapDiagnostics(result: CompileResult, doc: Text): CMLintDiag[] {
  return result.diagnostics.map((d) => {
    const startLine = doc.line(Math.min(d.span.start_line, doc.lines));
    const endLine = doc.line(Math.min(d.span.end_line, doc.lines));
    const startCharCol = byteColToCharCol(startLine.text, d.span.start_col);
    const endCharCol = byteColToCharCol(endLine.text, d.span.end_col);
    const from = startLine.from + Math.max(0, startCharCol - 1);
    const to = endLine.from + Math.max(0, endCharCol - 1);
    return {
      from: Math.max(0, from),
      to: Math.max(from + 1, to),
      severity: mapSeverity(d.severity),
      message: `[${d.code}] ${d.message}`,
    };
  });
}

function mapSeverity(s: string): 'error' | 'warning' | 'info' {
  if (s === 'error') return 'error';
  if (s === 'warning') return 'warning';
  return 'info';
}
