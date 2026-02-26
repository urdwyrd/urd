/**
 * Entity table projection — flat entity rows for table/list views.
 *
 * Depends on symbolTable, propertyDependencyIndex, urdJson.
 */

import type { ProjectionDefinition } from './ProjectionRegistry';
import type { ResolvedCompilerOutput } from '$lib/app/compiler/types';

export interface EntityRow {
  id: string;
  name: string;
  propertyCount: number;
  dependencyCount: number;
  file: string;
  line: number;
}

export const entityTableProjection: ProjectionDefinition<EntityRow[]> = {
  id: 'urd.projection.entityTable',
  depends: ['symbolTable', 'propertyDependencyIndex', 'urdJson'],
  compute: (source: ResolvedCompilerOutput): EntityRow[] => {
    const { symbolTable, propertyDependencyIndex, urdJson } = source;

    return urdJson.entities.map((entity) => {
      const symbol = symbolTable.entries.find((e) => e.id === entity.id);
      const deps = propertyDependencyIndex.dependencies.filter((d) =>
        d.property.startsWith(entity.id + '.')
      );

      return {
        id: entity.id,
        name: entity.name,
        propertyCount: Object.keys(entity.properties).length,
        dependencyCount: deps.length,
        file: symbol?.file ?? '',
        line: symbol?.line ?? 0,
      };
    });
  },
};
