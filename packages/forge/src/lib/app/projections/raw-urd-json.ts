/**
 * rawUrdJson projection — exposes the un-normalised compiler world JSON
 * for the hover tooltip, which uses object-keyed lookups matching the playground.
 */

import type { ProjectionDefinition } from './ProjectionRegistry';
import type { ResolvedCompilerOutput } from '$lib/app/compiler/types';

export type RawUrdJson = Record<string, unknown> | null;

export const rawUrdJsonProjection: ProjectionDefinition<RawUrdJson> = {
  id: 'urd.projection.rawUrdJson',
  depends: ['urdJson'],
  compute: (source: ResolvedCompilerOutput): RawUrdJson => {
    return source.rawUrdJson;
  },
};
