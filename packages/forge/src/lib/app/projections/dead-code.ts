/**
 * Dead code projection — finds unreferenced symbols.
 *
 * Uses the PropertyDependencyIndex orphan detection and cross-references
 * from the FactSet to find symbols that are never referenced.
 *
 * Depends on symbolTable, factSet, propertyDependencyIndex.
 */

import type { ProjectionDefinition } from './ProjectionRegistry';
import type { ResolvedCompilerOutput } from '$lib/app/compiler/types';

export interface DeadCodeEntry {
  id: string;
  name: string;
  kind: string;
  file: string;
  line: number;
  reason?: string;
}

export const deadCodeProjection: ProjectionDefinition<DeadCodeEntry[]> = {
  id: 'urd.projection.deadCode',
  depends: ['symbolTable', 'factSet', 'propertyDependencyIndex'],
  compute: (source: ResolvedCompilerOutput): DeadCodeEntry[] => {
    const { symbolTable, factSet, propertyDependencyIndex } = source;
    const entries: DeadCodeEntry[] = [];

    // Collect all referenced entity types from FactSet reads/writes
    const referencedTypes = new Set<string>();
    const referencedProperties = new Set<string>();

    for (const read of factSet.reads) {
      referencedTypes.add(read.entity_type);
      referencedProperties.add(`${read.entity_type}.${read.property}`);
    }
    for (const write of factSet.writes) {
      referencedTypes.add(write.entity_type);
      referencedProperties.add(`${write.entity_type}.${write.property}`);
    }

    // Collect referenced locations from exits
    const referencedLocations = new Set<string>();
    for (const exit of factSet.exits) {
      referencedLocations.add(exit.from_location);
      referencedLocations.add(exit.to_location);
    }

    // Collect referenced sections from jumps and choices
    const referencedSections = new Set<string>();
    for (const jump of factSet.jumps) {
      referencedSections.add(jump.from_section);
      if (jump.target.id) referencedSections.add(jump.target.id);
    }
    for (const choice of factSet.choices) {
      referencedSections.add(choice.section);
    }

    // Orphaned properties from PropertyDependencyIndex
    for (const prop of propertyDependencyIndex.properties) {
      if (prop.orphaned === 'read_never_written') {
        entries.push({
          id: `${prop.entity_type}.${prop.property}`,
          name: `${prop.entity_type}.${prop.property}`,
          kind: 'property',
          file: '',
          line: 0,
          reason: 'Read but never written',
        });
      } else if (prop.orphaned === 'written_never_read') {
        entries.push({
          id: `${prop.entity_type}.${prop.property}`,
          name: `${prop.entity_type}.${prop.property}`,
          kind: 'property',
          file: '',
          line: 0,
          reason: 'Written but never read',
        });
      }
    }

    // Find unreferenced symbols in the symbol table
    for (const sym of symbolTable.entries) {
      const isReferenced = (() => {
        switch (sym.kind) {
          case 'entity':
            // Entities are referenced if their type is used
            return true; // Entities are always "used" (they're instances)
          case 'location':
            return referencedLocations.has(sym.id) || referencedLocations.has(sym.name);
          case 'section':
            return referencedSections.has(sym.id) || referencedSections.has(sym.name);
          case 'type':
            return referencedTypes.has(sym.name);
          default:
            return true; // Don't flag unknown kinds
        }
      })();

      if (!isReferenced) {
        entries.push({
          id: sym.id,
          name: sym.name,
          kind: sym.kind,
          file: sym.file,
          line: sym.line,
        });
      }
    }

    return entries;
  },
};
