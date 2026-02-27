/**
 * Threshold analysis projection — flags numeric writes outside bounds.
 *
 * For each numeric property with min/max constraints, inspects FactSet
 * writes to find literal values that fall outside the declared range.
 *
 * Depends on urdJson, factSet.
 */

import type { ProjectionDefinition } from './ProjectionRegistry';
import type { ResolvedCompilerOutput } from '$lib/app/compiler/types';

export interface ThresholdEntry {
  entityType: string;
  property: string;
  min: number | null;
  max: number | null;
  suspiciousWrites: Array<{ value: string; file: string; line: number }>;
}

export const thresholdAnalysisProjection: ProjectionDefinition<ThresholdEntry[]> = {
  id: 'urd.projection.thresholdAnalysis',
  depends: ['urdJson', 'factSet'],
  compute: (source: ResolvedCompilerOutput): ThresholdEntry[] => {
    const { urdJson, factSet } = source;
    const entries: ThresholdEntry[] = [];

    if (!urdJson.types) return entries;

    // Build a lookup of numeric properties with bounds
    const numericProps = new Map<string, { min: number | null; max: number | null }>();

    for (const [typeName, typeDef] of Object.entries(urdJson.types)) {
      if (!typeDef.properties) continue;

      for (const [propName, propDef] of Object.entries(typeDef.properties)) {
        const hasMin = propDef.min !== undefined && propDef.min !== null;
        const hasMax = propDef.max !== undefined && propDef.max !== null;

        if (hasMin || hasMax) {
          const key = `${typeName}.${propName}`;
          numericProps.set(key, {
            min: hasMin ? propDef.min! : null,
            max: hasMax ? propDef.max! : null,
          });
        }
      }
    }

    if (numericProps.size === 0) return entries;

    // Check writes for suspicious values
    const suspiciousByKey = new Map<string, Array<{ value: string; file: string; line: number }>>();

    for (const write of factSet.writes) {
      const key = `${write.entity_type}.${write.property}`;
      const bounds = numericProps.get(key);
      if (!bounds) continue;

      const numericValue = parseFloat(write.value_expr);
      if (isNaN(numericValue)) continue;

      const outOfBounds =
        (bounds.min !== null && numericValue < bounds.min) ||
        (bounds.max !== null && numericValue > bounds.max);

      if (outOfBounds) {
        if (!suspiciousByKey.has(key)) suspiciousByKey.set(key, []);
        suspiciousByKey.get(key)!.push({
          value: write.value_expr,
          file: write.span.file,
          line: write.span.start_line,
        });
      }
    }

    // Build entries for all numeric properties with bounds
    for (const [key, bounds] of numericProps) {
      const [entityType, property] = key.split('.', 2);
      entries.push({
        entityType,
        property,
        min: bounds.min,
        max: bounds.max,
        suspiciousWrites: suspiciousByKey.get(key) ?? [],
      });
    }

    return entries;
  },
};
