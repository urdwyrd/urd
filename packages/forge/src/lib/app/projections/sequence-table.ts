/**
 * Sequence table projection — section sequence rows derived from jumps, choices, and symbols.
 *
 * Depends on factSet, symbolTable.
 */

import type { ProjectionDefinition } from './ProjectionRegistry';
import type { ResolvedCompilerOutput } from '$lib/app/compiler/types';

export interface SequenceRow {
  id: string;
  sectionId: string;
  sectionName: string;
  jumpCount: number;
  choiceCount: number;
  isTerminal: boolean;
  file: string;
  line: number;
}

export const sequenceTableProjection: ProjectionDefinition<SequenceRow[]> = {
  id: 'urd.projection.sequenceTable',
  depends: ['factSet', 'symbolTable'],
  compute: (source: ResolvedCompilerOutput): SequenceRow[] => {
    const { factSet, symbolTable } = source;

    // Collect all section IDs from jumps and choices
    const sectionIds = new Set<string>();
    for (const jump of factSet.jumps) sectionIds.add(jump.from_section);
    for (const choice of factSet.choices) sectionIds.add(choice.section);

    // Also include sections from the symbol table
    for (const sym of symbolTable.entries) {
      if (sym.kind === 'section') sectionIds.add(sym.id);
    }

    return Array.from(sectionIds).map((sectionId) => {
      const sym = symbolTable.entries.find((e) => e.id === sectionId || e.name === sectionId);
      const jumpsFrom = factSet.jumps.filter((j) => j.from_section === sectionId);
      const choicesIn = factSet.choices.filter((c) => c.section === sectionId);
      const hasOutgoing = jumpsFrom.length > 0 || choicesIn.some((c) => c.jump_indices.length > 0);

      return {
        id: sectionId,
        sectionId,
        sectionName: sym?.name ?? sectionId,
        jumpCount: jumpsFrom.length,
        choiceCount: choicesIn.length,
        isTerminal: !hasOutgoing,
        file: sym?.file ?? '',
        line: sym?.line ?? 0,
      };
    });
  },
};
