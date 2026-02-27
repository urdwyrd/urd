/**
 * Write table projection — flat property write rows for table/list views.
 *
 * Depends on factSet.
 */

import type { ProjectionDefinition } from './ProjectionRegistry';
import type { ResolvedCompilerOutput } from '$lib/app/compiler/types';

export interface WriteRow {
  id: string;
  siteKind: string;
  siteId: string;
  entityType: string;
  property: string;
  operator: string;
  valueExpr: string;
  file: string;
  line: number;
}

export const writeTableProjection: ProjectionDefinition<WriteRow[]> = {
  id: 'urd.projection.writeTable',
  depends: ['factSet'],
  compute: (source: ResolvedCompilerOutput): WriteRow[] => {
    const { factSet } = source;

    return factSet.writes.map((write, i) => ({
      id: `write-${i}-${write.site.id}-${write.property}`,
      siteKind: write.site.kind,
      siteId: write.site.id,
      entityType: write.entity_type,
      property: write.property,
      operator: write.operator,
      valueExpr: write.value_expr,
      file: write.span.file,
      line: write.span.start_line,
    }));
  },
};
