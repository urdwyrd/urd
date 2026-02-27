/**
 * Choice table projection — flat choice rows for table/list views.
 *
 * Depends on factSet.
 */

import type { ProjectionDefinition } from './ProjectionRegistry';
import type { ResolvedCompilerOutput } from '$lib/app/compiler/types';

export interface ChoiceRow {
  id: string;
  section: string;
  label: string;
  sticky: boolean;
  conditionCount: number;
  effectCount: number;
  jumpCount: number;
  file: string;
  line: number;
}

export const choiceTableProjection: ProjectionDefinition<ChoiceRow[]> = {
  id: 'urd.projection.choiceTable',
  depends: ['factSet'],
  compute: (source: ResolvedCompilerOutput): ChoiceRow[] => {
    const { factSet } = source;

    return factSet.choices.map((choice) => ({
      id: choice.choice_id,
      section: choice.section,
      label: choice.label,
      sticky: choice.sticky,
      conditionCount: choice.condition_reads.length,
      effectCount: choice.effect_writes.length,
      jumpCount: choice.jump_indices.length,
      file: choice.span.file,
      line: choice.span.start_line,
    }));
  },
};
