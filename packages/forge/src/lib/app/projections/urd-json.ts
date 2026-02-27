/**
 * urdJson passthrough projection — exposes the resolved UrdWorld data
 * through the projection registry for editor extensions (hover, autocomplete, etc.).
 */

import type { ProjectionDefinition } from './ProjectionRegistry';
import type { ResolvedCompilerOutput, UrdWorld } from '$lib/app/compiler/types';

export const urdJsonProjection: ProjectionDefinition<UrdWorld> = {
  id: 'urd.projection.urdJson',
  depends: ['urdJson'],
  compute: (source: ResolvedCompilerOutput): UrdWorld => {
    return source.urdJson;
  },
};
