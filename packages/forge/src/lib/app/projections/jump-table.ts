/**
 * Jump table projection — flat jump edge rows for table/list views.
 *
 * Depends on factSet.
 */

import type { ProjectionDefinition } from './ProjectionRegistry';
import type { ResolvedCompilerOutput } from '$lib/app/compiler/types';

export interface JumpRow {
  id: string;
  fromSection: string;
  targetKind: string;
  targetId: string;
  file: string;
  line: number;
}

export const jumpTableProjection: ProjectionDefinition<JumpRow[]> = {
  id: 'urd.projection.jumpTable',
  depends: ['factSet'],
  compute: (source: ResolvedCompilerOutput): JumpRow[] => {
    const { factSet } = source;

    return factSet.jumps.map((jump, i) => ({
      id: `jump-${i}-${jump.from_section}`,
      fromSection: jump.from_section,
      targetKind: jump.target.kind,
      targetId: jump.target.id ?? '',
      file: jump.span.file,
      line: jump.span.start_line,
    }));
  },
};
