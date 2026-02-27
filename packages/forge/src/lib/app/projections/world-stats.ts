/**
 * World stats projection — aggregate counts for the dashboard.
 *
 * Depends on all chunk types for comprehensive statistics.
 */

import type { ProjectionDefinition } from './ProjectionRegistry';
import type { ResolvedCompilerOutput } from '$lib/app/compiler/types';

export interface WorldStats {
  entityCount: number;
  locationCount: number;
  exitCount: number;
  propertyCount: number;
  ruleCount: number;
  factCount: number;
  symbolCount: number;
  diagnosticCount: number;
  errorCount: number;
  warningCount: number;
  infoCount: number;
  fileCount: number;
  compileDurationMs: number;
  phaseTimings: { phase: string; durationMs: number }[];
}

export const worldStatsProjection: ProjectionDefinition<WorldStats> = {
  id: 'urd.projection.worldStats',
  depends: ['symbolTable', 'factSet', 'urdJson', 'diagnostics'],
  compute: (source: ResolvedCompilerOutput): WorldStats => {
    const { header, symbolTable, factSet, urdJson, diagnostics } = source;

    const files = new Set<string>();
    for (const sym of symbolTable.entries) {
      if (sym.file) files.add(sym.file);
    }

    return {
      entityCount: header.worldCounts.entities,
      locationCount: header.worldCounts.locations,
      exitCount: header.worldCounts.exits,
      propertyCount: header.worldCounts.properties,
      ruleCount: header.worldCounts.rules,
      factCount: factSet.reads.length + factSet.writes.length + factSet.exits.length +
        factSet.jumps.length + factSet.choices.length + factSet.rules.length,
      symbolCount: symbolTable.entries.length,
      diagnosticCount: diagnostics.length,
      errorCount: diagnostics.filter((d) => d.severity === 'error').length,
      warningCount: diagnostics.filter((d) => d.severity === 'warning').length,
      infoCount: diagnostics.filter((d) => d.severity === 'info').length,
      fileCount: files.size || header.inputFileCount,
      compileDurationMs: header.durationMs,
      phaseTimings: header.phaseTimings.map((p) => ({ phase: p.phase, durationMs: p.durationMs })),
    };
  },
};
