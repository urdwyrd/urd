/**
 * Outline projection — symbols grouped by file, sorted by line.
 *
 * Depends on symbolTable.
 */

import type { ProjectionDefinition } from './ProjectionRegistry';
import type { ResolvedCompilerOutput } from '$lib/app/compiler/types';

export interface OutlineEntry {
  id: string;
  name: string;
  kind: string;
  file: string;
  line: number;
}

export interface FileOutline {
  file: string;
  entries: OutlineEntry[];
}

export const outlineProjection: ProjectionDefinition<FileOutline[]> = {
  id: 'urd.projection.outline',
  depends: ['symbolTable'],
  compute: (source: ResolvedCompilerOutput): FileOutline[] => {
    const { symbolTable } = source;

    const grouped = new Map<string, OutlineEntry[]>();

    for (const sym of symbolTable.entries) {
      const file = sym.file || '(unknown)';
      const list = grouped.get(file) ?? [];
      list.push({
        id: sym.id,
        name: sym.name,
        kind: sym.kind,
        file: sym.file,
        line: sym.line,
      });
      grouped.set(file, list);
    }

    // Sort entries within each file by line number
    const result: FileOutline[] = [];
    for (const [file, entries] of grouped) {
      entries.sort((a, b) => a.line - b.line);
      result.push({ file, entries });
    }

    // Sort files alphabetically
    result.sort((a, b) => a.file.localeCompare(b.file));
    return result;
  },
};
