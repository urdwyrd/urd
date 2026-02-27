/**
 * factSet passthrough projection — exposes the full FactSet data
 * through the projection registry for editor extensions (hover tooltips, etc.).
 */

import type { ProjectionDefinition } from './ProjectionRegistry';
import type { ResolvedCompilerOutput, FactSet } from '$lib/app/compiler/types';

export const factSetProjection: ProjectionDefinition<FactSet> = {
  id: 'urd.projection.factSet',
  depends: ['factSet'],
  compute: (source: ResolvedCompilerOutput): FactSet => {
    return source.factSet;
  },
};
