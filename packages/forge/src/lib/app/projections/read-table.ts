/**
 * Read table projection — flat property read rows for table/list views.
 *
 * Depends on factSet.
 */

import type { ProjectionDefinition } from './ProjectionRegistry';
import type { ResolvedCompilerOutput } from '$lib/app/compiler/types';

export interface ReadRow {
  id: string;
  siteKind: string;
  siteId: string;
  entityType: string;
  property: string;
  operator: string;
  valueLiteral: string;
  file: string;
  line: number;
}

export const readTableProjection: ProjectionDefinition<ReadRow[]> = {
  id: 'urd.projection.readTable',
  depends: ['factSet'],
  compute: (source: ResolvedCompilerOutput): ReadRow[] => {
    const { factSet } = source;

    return factSet.reads.map((read, i) => ({
      id: `read-${i}-${read.site.id}-${read.property}`,
      siteKind: read.site.kind,
      siteId: read.site.id,
      entityType: read.entity_type,
      property: read.property,
      operator: read.operator,
      valueLiteral: read.value_literal,
      file: read.span.file,
      line: read.span.start_line,
    }));
  },
};
