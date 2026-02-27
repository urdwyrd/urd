/**
 * File table projection — file-level summary rows from symbol table and diagnostics.
 *
 * Depends on symbolTable, diagnostics.
 */

import type { ProjectionDefinition } from './ProjectionRegistry';
import type { ResolvedCompilerOutput } from '$lib/app/compiler/types';

export interface FileRow {
  id: string;
  filePath: string;
  symbolCount: number;
  errorCount: number;
  warningCount: number;
}

export const fileTableProjection: ProjectionDefinition<FileRow[]> = {
  id: 'urd.projection.fileTable',
  depends: ['symbolTable', 'diagnostics'],
  compute: (source: ResolvedCompilerOutput): FileRow[] => {
    const { symbolTable, diagnostics } = source;

    // Group symbols by file
    const fileMap = new Map<string, { symbols: number; errors: number; warnings: number }>();

    for (const sym of symbolTable.entries) {
      if (!sym.file) continue;
      const entry = fileMap.get(sym.file) ?? { symbols: 0, errors: 0, warnings: 0 };
      entry.symbols++;
      fileMap.set(sym.file, entry);
    }

    // Add diagnostic counts
    for (const diag of diagnostics) {
      const file = diag.span?.file;
      if (!file) continue;
      const entry = fileMap.get(file) ?? { symbols: 0, errors: 0, warnings: 0 };
      if (diag.severity === 'error') entry.errors++;
      else if (diag.severity === 'warning') entry.warnings++;
      fileMap.set(file, entry);
    }

    return Array.from(fileMap.entries()).map(([filePath, counts]) => ({
      id: filePath,
      filePath,
      symbolCount: counts.symbols,
      errorCount: counts.errors,
      warningCount: counts.warnings,
    }));
  },
};
