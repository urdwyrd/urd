/**
 * Property table projection — flat property rows for table views.
 *
 * Derives instance-level property values from urdJson entities and enriches
 * with read/write counts from the propertyDependencyIndex.
 *
 * Depends on urdJson, symbolTable, propertyDependencyIndex, definitionIndex.
 */

import type { ProjectionDefinition } from './ProjectionRegistry';
import type { ResolvedCompilerOutput } from '$lib/app/compiler/types';

export interface PropertyRow {
  subject: string;
  subjectName: string;
  predicate: string;
  object: string;
  dependencyCount: number;
  readCount: number;
  writeCount: number;
  file: string;
  line: number;
}

export const propertyTableProjection: ProjectionDefinition<PropertyRow[]> = {
  id: 'urd.projection.propertyTable',
  depends: ['urdJson', 'symbolTable', 'propertyDependencyIndex', 'definitionIndex'],
  compute: (source: ResolvedCompilerOutput): PropertyRow[] => {
    const { urdJson, symbolTable, propertyDependencyIndex, definitionIndex } = source;
    const rows: PropertyRow[] = [];

    for (const entity of urdJson.entities) {
      const entitySymbol = symbolTable.entries.find(
        (e) => e.id === entity.id || e.name === entity.name || e.id === entity.name,
      );
      const entityType = (entity as Record<string, unknown>).type as string | undefined;

      for (const [key, val] of Object.entries(entity.properties)) {
        const objectStr = val === null || val === undefined
          ? ''
          : typeof val === 'string' ? val : String(val);

        // Find read/write counts from propertyDependencyIndex
        const propEntry = entityType
          ? propertyDependencyIndex.properties.find(
              (p) => p.entity_type === entityType && p.property === key,
            )
          : undefined;

        // Find file/line from definition index
        const defKey = entityType ? `prop:${entityType}.${key}` : undefined;
        const defEntry = defKey
          ? definitionIndex.definitions.find((d) => d.key === defKey)
          : undefined;

        rows.push({
          subject: entity.id,
          subjectName: entitySymbol?.name ?? entity.name ?? entity.id,
          predicate: key,
          object: objectStr,
          dependencyCount: (propEntry?.read_count ?? 0) + (propEntry?.write_count ?? 0),
          readCount: propEntry?.read_count ?? 0,
          writeCount: propEntry?.write_count ?? 0,
          file: defEntry?.span.file ?? entitySymbol?.file ?? '',
          line: defEntry?.span.start_line ?? entitySymbol?.line ?? 0,
        });
      }
    }

    return rows;
  },
};
