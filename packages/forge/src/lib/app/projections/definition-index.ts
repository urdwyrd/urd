/**
 * Definition index projection — exposes the compiler's rich DefinitionIndex
 * for go-to-definition, hover resolution, and symbol lookup.
 *
 * Depends on definitionIndex chunk (from the Rust compiler's DefinitionIndex::to_json()).
 * Falls back to symbolTable chunk for backward compatibility with mock/fixture data.
 */

import type { ProjectionDefinition } from './ProjectionRegistry';
import type {
  ResolvedCompilerOutput,
  SymbolTableEntry,
  RichDefinitionEntry,
} from '$lib/app/compiler/types';

/** Re-export the rich entry type for consumers. */
export type DefinitionEntry = RichDefinitionEntry;

export type DefinitionIndex = DefinitionEntry[];

export const definitionIndexProjection: ProjectionDefinition<DefinitionIndex> = {
  id: 'urd.projection.definitionIndex',
  depends: ['definitionIndex', 'symbolTable'],
  compute: (source: ResolvedCompilerOutput): DefinitionIndex => {
    // Prefer the rich definitionIndex chunk from the compiler
    if (source.definitionIndex && source.definitionIndex.definitions.length > 0) {
      if (import.meta.env?.DEV) {
        const defs = source.definitionIndex.definitions;
        const propEntries = defs.filter((d) => d.key.startsWith('prop:'));
        console.warn('[definitionIndex] RICH path:', defs.length, 'entries,', propEntries.length, 'properties',
          'sample:', JSON.stringify(defs[0]),
          'propSample:', propEntries.length > 0 ? JSON.stringify(propEntries[0]) : 'none');
      }
      return source.definitionIndex.definitions;
    }

    // Fallback: build from flat symbolTable (mock/fixture data)
    if (import.meta.env?.DEV) {
      console.warn('[definitionIndex] FALLBACK path: symbolTable entries:', source.symbolTable.entries.length,
        'definitionIndex present:', !!source.definitionIndex,
        'definitions length:', source.definitionIndex?.definitions?.length ?? 'N/A');
    }
    const entries: DefinitionEntry[] = [];
    for (const sym of source.symbolTable.entries) {
      const entry = symbolEntryToDefinition(sym);
      if (entry) {
        entries.push(entry);
      }
    }
    return entries;
  },
};

function symbolEntryToDefinition(sym: SymbolTableEntry): DefinitionEntry | null {
  const span = {
    file: sym.file,
    start_line: sym.line,
    start_col: 1,
    end_line: sym.line,
    end_col: sym.name.length + 1,
  };

  switch (sym.kind) {
    case 'entity':
      return {
        key: `entity:@${sym.name}`,
        span,
        definition: { kind: 'entity', display_name: sym.name },
      };
    case 'location':
      return {
        key: `location:${sym.name}`,
        span,
        definition: { kind: 'location', display_name: sym.name },
      };
    case 'section':
      return {
        key: `section:${sym.name}`,
        span,
        definition: { kind: 'section', local_name: sym.name, display_name: sym.name },
      };
    case 'type':
      return {
        key: `type:${sym.name}`,
        span,
        definition: { kind: 'type', display_name: sym.name },
      };
    case 'property':
      return {
        key: `prop:${sym.name}`,
        span,
        definition: { kind: 'property', display_name: sym.name },
      };
    default:
      return {
        key: `${sym.kind}:${sym.name}`,
        span,
        definition: { kind: sym.kind, display_name: sym.name },
      };
  }
}
