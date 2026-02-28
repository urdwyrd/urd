/**
 * Global filter logic for World Insight panel.
 *
 * Performs case-insensitive substring matching across all section data,
 * returning sets of matching IDs per section.
 */

import type { InsightState } from './insight-state';

export interface FilterResult {
  entityMatches: Set<string>;
  propertyMatches: Set<string>;
  sectionMatches: Set<string>;
  locationMatches: Set<string>;
  ruleMatches: Set<string>;
  diagnosticMatches: Set<string>;
}

export function computeFilter(filterText: string, state: InsightState): FilterResult {
  const q = filterText.toLowerCase();
  const result: FilterResult = {
    entityMatches: new Set(),
    propertyMatches: new Set(),
    sectionMatches: new Set(),
    locationMatches: new Set(),
    ruleMatches: new Set(),
    diagnosticMatches: new Set(),
  };

  if (!q) return result;

  // Entities
  if (state.world) {
    for (const entity of state.world.entities) {
      if (
        entity.id.toLowerCase().includes(q) ||
        entity.name.toLowerCase().includes(q) ||
        (entity.type ?? '').toLowerCase().includes(q) ||
        Object.entries(entity.properties).some(
          ([k, v]) => k.toLowerCase().includes(q) || String(v).toLowerCase().includes(q),
        )
      ) {
        result.entityMatches.add(entity.id);
      }
    }
  }

  // Properties
  if (state.propertyIndex) {
    for (const prop of state.propertyIndex.properties) {
      const key = `${prop.entity_type}.${prop.property}`;
      if (key.toLowerCase().includes(q)) {
        result.propertyMatches.add(key);
      }
    }
  }

  // Sections (from choices and jumps)
  if (state.factSet) {
    for (const choice of state.factSet.choices) {
      if (
        choice.section.toLowerCase().includes(q) ||
        choice.label.toLowerCase().includes(q)
      ) {
        result.sectionMatches.add(choice.section);
      }
    }
    for (const jump of state.factSet.jumps) {
      if (jump.from_section.toLowerCase().includes(q)) {
        result.sectionMatches.add(jump.from_section);
      }
    }
  }

  // Locations
  if (state.world) {
    for (const loc of state.world.locations) {
      if (
        loc.id.toLowerCase().includes(q) ||
        loc.name.toLowerCase().includes(q) ||
        loc.exits.some(
          (e) =>
            e.direction.toLowerCase().includes(q) ||
            e.target.toLowerCase().includes(q),
        )
      ) {
        result.locationMatches.add(loc.id);
      }
    }
  }

  // Rules
  if (state.world?.rules) {
    for (const ruleId of Object.keys(state.world.rules)) {
      if (ruleId.toLowerCase().includes(q)) {
        result.ruleMatches.add(ruleId);
      }
    }
  }

  // Diagnostics
  if (state.diagnosticsByFile) {
    for (const fileDiag of state.diagnosticsByFile) {
      for (const diag of fileDiag.diagnostics) {
        if (
          diag.message.toLowerCase().includes(q) ||
          diag.code.toLowerCase().includes(q)
        ) {
          const key = `${diag.span?.file ?? fileDiag.file}:${diag.span?.startLine ?? 0}`;
          result.diagnosticMatches.add(key);
        }
      }
    }
  }

  return result;
}

export function hasAnyMatches(result: FilterResult): boolean {
  return (
    result.entityMatches.size > 0 ||
    result.propertyMatches.size > 0 ||
    result.sectionMatches.size > 0 ||
    result.locationMatches.size > 0 ||
    result.ruleMatches.size > 0 ||
    result.diagnosticMatches.size > 0
  );
}
