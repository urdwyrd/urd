/**
 * Location table projection — flat location rows for table views.
 *
 * Depends on symbolTable, urdJson.
 */

import type { ProjectionDefinition } from './ProjectionRegistry';
import type { ResolvedCompilerOutput } from '$lib/app/compiler/types';

export interface LocationRow {
  id: string;
  name: string;
  description: string;
  exitCount: number;
  file: string;
  line: number;
}

export const locationTableProjection: ProjectionDefinition<LocationRow[]> = {
  id: 'urd.projection.locationTable',
  depends: ['symbolTable', 'urdJson'],
  compute: (source: ResolvedCompilerOutput): LocationRow[] => {
    const { symbolTable, urdJson } = source;

    return urdJson.locations.map((loc) => {
      const symbol = symbolTable.entries.find((e) => e.id === loc.id || e.name === loc.name);

      return {
        id: loc.id,
        name: symbol?.name ?? loc.name,
        description: loc.description,
        exitCount: loc.exits.length,
        file: symbol?.file ?? '',
        line: symbol?.line ?? 0,
      };
    });
  },
};
