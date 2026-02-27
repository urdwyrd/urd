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
  type: string;
  propertyCount: number;
  dependencyCount: number;
  readCount: number;
  writeCount: number;
  file: string;
  line: number;
}

export const entityTableProjection: ProjectionDefinition<EntityRow[]> = {
  id: 'urd.projection.entityTable',
  depends: ['symbolTable', 'propertyDependencyIndex', 'urdJson'],
  compute: (source: ResolvedCompilerOutput): EntityRow[] => {
    const { symbolTable, propertyDependencyIndex, urdJson } = source;

    return urdJson.entities.map((entity) => {
      const symbol = symbolTable.entries.find((e) => e.id === entity.id || e.name === entity.name);
      const entityType = (entity as Record<string, unknown>).type as string | undefined;

      // Count dependencies from the PropertyDependencyIndex for this entity's type
      const deps = entityType
        ? propertyDependencyIndex.properties.filter((p) => p.entity_type === entityType)
        : [];
      const totalReads = deps.reduce((sum, d) => sum + d.read_count, 0);
      const totalWrites = deps.reduce((sum, d) => sum + d.write_count, 0);

      return {
        id: entity.id,
        name: symbol?.name ?? entity.name,
        type: entityType ?? '',
        propertyCount: Object.keys(entity.properties).length,
        dependencyCount: deps.length,
        readCount: totalReads,
        writeCount: totalWrites,
        file: symbol?.file ?? '',
        line: symbol?.line ?? 0,
      };
    });
  },
};
