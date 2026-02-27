/**
 * Exit table projection — flat exit edge rows for table/list views.
 *
 * Depends on factSet.
 */

import type { ProjectionDefinition } from './ProjectionRegistry';
import type { ResolvedCompilerOutput } from '$lib/app/compiler/types';

export interface ExitRow {
  id: string;
  fromLocation: string;
  toLocation: string;
  exitName: string;
  isConditional: boolean;
  guardCount: number;
  file: string;
  line: number;
}

export const exitTableProjection: ProjectionDefinition<ExitRow[]> = {
  id: 'urd.projection.exitTable',
  depends: ['factSet'],
  compute: (source: ResolvedCompilerOutput): ExitRow[] => {
    const { factSet } = source;

    return factSet.exits.map((exit, i) => ({
      id: `exit-${i}-${exit.from_location}-${exit.exit_name}`,
      fromLocation: exit.from_location,
      toLocation: exit.to_location,
      exitName: exit.exit_name,
      isConditional: exit.is_conditional,
      guardCount: exit.guard_reads.length,
      file: exit.span.file,
      line: exit.span.start_line,
    }));
  },
};
