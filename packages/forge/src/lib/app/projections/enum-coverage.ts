/**
 * Enum coverage projection — tracks which enum values are used.
 *
 * For each enum property (types with a `values` array), checks which
 * values appear in FactSet reads and writes. Produces coverage metrics
 * so authors can spot unused enum branches.
 *
 * Depends on urdJson, factSet.
 */

import type { ProjectionDefinition } from './ProjectionRegistry';
import type { ResolvedCompilerOutput } from '$lib/app/compiler/types';

export interface EnumCoverageEntry {
  entityType: string;
  property: string;
  totalValues: number;
  usedValues: string[];
  unusedValues: string[];
  coveragePct: number;
}

export const enumCoverageProjection: ProjectionDefinition<EnumCoverageEntry[]> = {
  id: 'urd.projection.enumCoverage',
  depends: ['urdJson', 'factSet'],
  compute: (source: ResolvedCompilerOutput): EnumCoverageEntry[] => {
    const { urdJson, factSet } = source;
    const entries: EnumCoverageEntry[] = [];

    if (!urdJson.types) return entries;

    // Build a set of observed values per entity_type.property from reads/writes
    const observedValues = new Map<string, Set<string>>();

    for (const read of factSet.reads) {
      const key = `${read.entity_type}.${read.property}`;
      if (!observedValues.has(key)) observedValues.set(key, new Set());
      if (read.value_literal) {
        observedValues.get(key)!.add(read.value_literal);
      }
    }

    for (const write of factSet.writes) {
      const key = `${write.entity_type}.${write.property}`;
      if (!observedValues.has(key)) observedValues.set(key, new Set());
      if (write.value_expr) {
        observedValues.get(key)!.add(write.value_expr);
      }
    }

    // Walk types and find enum properties (those with a values array)
    for (const [typeName, typeDef] of Object.entries(urdJson.types)) {
      if (!typeDef.properties) continue;

      for (const [propName, propDef] of Object.entries(typeDef.properties)) {
        if (!propDef.values || propDef.values.length === 0) continue;

        const key = `${typeName}.${propName}`;
        const observed = observedValues.get(key) ?? new Set<string>();
        const allValues = propDef.values;
        const usedValues = allValues.filter((v) => observed.has(v));
        const unusedValues = allValues.filter((v) => !observed.has(v));
        const coveragePct = allValues.length > 0
          ? Math.round((usedValues.length / allValues.length) * 100)
          : 100;

        entries.push({
          entityType: typeName,
          property: propName,
          totalValues: allValues.length,
          usedValues,
          unusedValues,
          coveragePct,
        });
      }
    }

    return entries;
  },
};
