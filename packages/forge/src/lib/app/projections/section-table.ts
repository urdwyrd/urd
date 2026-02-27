/**
 * Section table projection — flat section rows for table views.
 *
 * Depends on symbolTable.
 */

import type { ProjectionDefinition } from './ProjectionRegistry';
import type { ResolvedCompilerOutput } from '$lib/app/compiler/types';

export interface SectionRow {
  id: string;
  name: string;
  file: string;
  line: number;
}

export const sectionTableProjection: ProjectionDefinition<SectionRow[]> = {
  id: 'urd.projection.sectionTable',
  depends: ['symbolTable'],
  compute: (source: ResolvedCompilerOutput): SectionRow[] => {
    const { symbolTable } = source;

    return symbolTable.entries
      .filter((e) => e.kind === 'section')
      .map((sym) => ({
        id: sym.id,
        name: sym.name,
        file: sym.file,
        line: sym.line,
      }));
  },
};
