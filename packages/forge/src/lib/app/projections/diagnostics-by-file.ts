/**
 * Diagnostics-by-file projection — groups diagnostics by source file path.
 *
 * Depends on diagnostics chunk.
 */

import type { ProjectionDefinition } from './ProjectionRegistry';
import type { ResolvedCompilerOutput, Diagnostic } from '$lib/app/compiler/types';

export interface FileDiagnostics {
  file: string;
  diagnostics: Diagnostic[];
  errorCount: number;
  warningCount: number;
  infoCount: number;
}

export const diagnosticsByFileProjection: ProjectionDefinition<FileDiagnostics[]> = {
  id: 'urd.projection.diagnosticsByFile',
  depends: ['diagnostics'],
  compute: (source: ResolvedCompilerOutput): FileDiagnostics[] => {
    const grouped = new Map<string, Diagnostic[]>();

    for (const diag of source.diagnostics) {
      const file = diag.span?.file ?? '(no file)';
      const list = grouped.get(file) ?? [];
      list.push(diag);
      grouped.set(file, list);
    }

    return Array.from(grouped.entries())
      .map(([file, diagnostics]) => ({
        file,
        diagnostics,
        errorCount: diagnostics.filter((d) => d.severity === 'error').length,
        warningCount: diagnostics.filter((d) => d.severity === 'warning').length,
        infoCount: diagnostics.filter((d) => d.severity === 'info').length,
      }))
      .sort((a, b) => a.file.localeCompare(b.file));
  },
};
