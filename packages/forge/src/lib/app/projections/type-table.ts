/**
 * Type table projection — flat type rows for table views.
 *
 * Depends on symbolTable, urdJson.
 */

import type { ProjectionDefinition } from './ProjectionRegistry';
import type { ResolvedCompilerOutput } from '$lib/app/compiler/types';

export interface TypeRow {
  id: string;
  name: string;
  entityCount: number;
  propertyCount: number;
  file: string;
  line: number;
}

export const typeTableProjection: ProjectionDefinition<TypeRow[]> = {
  id: 'urd.projection.typeTable',
  depends: ['symbolTable', 'urdJson'],
  compute: (source: ResolvedCompilerOutput): TypeRow[] => {
    const { symbolTable, urdJson } = source;

    const typeSymbols = symbolTable.entries.filter((e) => e.kind === 'type');

    return typeSymbols.map((sym) => {
      // Count entities whose properties reference this type
      const entityCount = urdJson.entities.filter((ent) => {
        const props = ent.properties as Record<string, unknown>;
        return Object.values(props).some((v) => v === sym.name || v === sym.id);
      }).length;

      // Count properties associated with this type from the symbol table
      const propertyCount = symbolTable.entries.filter(
        (e) => e.kind === 'property' && e.name.startsWith(sym.name + '.')
      ).length;

      return {
        id: sym.id,
        name: sym.name,
        entityCount,
        propertyCount,
        file: sym.file,
        line: sym.line,
      };
    });
  },
};
