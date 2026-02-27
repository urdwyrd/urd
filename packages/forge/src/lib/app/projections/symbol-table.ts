/**
 * symbolTable passthrough projection — exposes the resolved SymbolTable data
 * through the projection registry for editor extensions (hover, go-to-definition, etc.).
 */

import type { ProjectionDefinition } from './ProjectionRegistry';
import type { ResolvedCompilerOutput, SymbolTable } from '$lib/app/compiler/types';

export const symbolTableProjection: ProjectionDefinition<SymbolTable> = {
  id: 'urd.projection.symbolTable',
  depends: ['symbolTable'],
  compute: (source: ResolvedCompilerOutput): SymbolTable => {
    return source.symbolTable;
  },
};
