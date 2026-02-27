/**
 * Visibility audit projection — properties with visibility restrictions.
 *
 * Lists all properties that have visibility constraints and counts how
 * many reads and writes target them. Helps authors verify that visibility
 * rules are respected across the world definition.
 *
 * Depends on urdJson, propertyDependencyIndex.
 */

import type { ProjectionDefinition } from './ProjectionRegistry';
import type { ResolvedCompilerOutput } from '$lib/app/compiler/types';

export interface VisibilityEntry {
  entityType: string;
  property: string;
  visibility: string;
  readCount: number;
  writeCount: number;
}

export const visibilityAuditProjection: ProjectionDefinition<VisibilityEntry[]> = {
  id: 'urd.projection.visibilityAudit',
  depends: ['urdJson', 'propertyDependencyIndex'],
  compute: (source: ResolvedCompilerOutput): VisibilityEntry[] => {
    const { urdJson, propertyDependencyIndex } = source;
    const entries: VisibilityEntry[] = [];

    if (!urdJson.types) return entries;

    // Build a lookup from property dependency index for read/write counts
    const depLookup = new Map<string, { readCount: number; writeCount: number }>();
    for (const dep of propertyDependencyIndex.properties) {
      const key = `${dep.entity_type}.${dep.property}`;
      depLookup.set(key, {
        readCount: dep.read_count,
        writeCount: dep.write_count,
      });
    }

    // Walk all type properties looking for visibility restrictions
    for (const [typeName, typeDef] of Object.entries(urdJson.types)) {
      if (!typeDef.properties) continue;

      for (const [propName, propDef] of Object.entries(typeDef.properties)) {
        if (!propDef.visibility) continue;

        const key = `${typeName}.${propName}`;
        const counts = depLookup.get(key) ?? { readCount: 0, writeCount: 0 };

        entries.push({
          entityType: typeName,
          property: propName,
          visibility: propDef.visibility,
          readCount: counts.readCount,
          writeCount: counts.writeCount,
        });
      }
    }

    return entries;
  },
};
