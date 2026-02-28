/**
 * compilerOutput projection — exposes the full resolved compiler output
 * for the Compiler Output panel (JSON viewer).
 */

import type { ProjectionDefinition } from './ProjectionRegistry';
import type { ResolvedCompilerOutput } from '$lib/app/compiler/types';

export const compilerOutputProjection: ProjectionDefinition<ResolvedCompilerOutput> = {
  id: 'urd.projection.compilerOutput',
  depends: ['ast', 'symbolTable', 'factSet', 'propertyDependencyIndex', 'definitionIndex', 'urdJson', 'diagnostics'],
  compute: (source: ResolvedCompilerOutput): ResolvedCompilerOutput => {
    return source;
  },
};
