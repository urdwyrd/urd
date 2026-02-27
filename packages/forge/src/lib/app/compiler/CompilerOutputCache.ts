/**
 * Compiler output cache — resolves chunked output into a typed structure.
 *
 * One entry per chunk name. If a chunk's content hash hasn't changed,
 * the cached resolved value is reused. This avoids re-parsing large
 * chunks that didn't change between compiles.
 */

import type {
  CompilerOutput,
  ResolvedCompilerOutput,
  Chunk,
  ChunkName,
  AST,
  SymbolTable,
  FactSet,
  PropertyDependencyIndex,
  RichDefinitionIndex,
  UrdWorld,
  UrdWorldMeta,
  UrdTypeDef,
  UrdEntity,
  UrdLocation,
  UrdExit,
  Diagnostic,
} from './types';

interface CacheEntry {
  contentHash: string;
  resolved: unknown;
}

/**
 * Normalise the urdJson chunk from the compiler's object-keyed format to
 * the array-based UrdWorld shape expected by projections.
 *
 * The Rust emitter outputs entities/locations as `{ "id": {...} }` maps.
 * The fixture and UrdWorld type expect `[{ id, name, ... }]` arrays.
 * If the data is already in array form (e.g. from the fixture/mock), pass through.
 */
function normaliseUrdJson(raw: unknown): UrdWorld {
  const data = raw as Record<string, unknown> | null;
  if (!data) return { entities: [], locations: [] };

  const entities = normaliseEntities(data.entities);
  const locations = normaliseLocations(data.locations);

  const result: UrdWorld = { entities, locations };

  // Preserve world metadata
  if (data.world && typeof data.world === 'object') {
    result.world = data.world as UrdWorldMeta;
  }

  // Preserve type definitions (object-keyed, no normalisation needed)
  if (data.types && typeof data.types === 'object') {
    result.types = data.types as Record<string, UrdTypeDef>;
  }

  return result;
}

function normaliseEntities(raw: unknown): UrdEntity[] {
  if (!raw) return [];
  // Already an array (fixture/mock format)
  if (Array.isArray(raw)) return raw as UrdEntity[];
  // Object keyed by id (real compiler format)
  if (typeof raw === 'object') {
    return Object.entries(raw as Record<string, unknown>).map(([id, val]) => {
      const obj = val as Record<string, unknown>;
      const entity: UrdEntity = {
        id,
        name: (obj.name as string) ?? id,
        properties: (obj.properties as Record<string, unknown>) ?? {},
      };
      if (obj.type) entity.type = obj.type as string;
      if (obj.contains) entity.contains = obj.contains as string[];
      return entity;
    });
  }
  return [];
}

function normaliseLocations(raw: unknown): UrdLocation[] {
  if (!raw) return [];
  // Already an array (fixture/mock format)
  if (Array.isArray(raw)) return raw as UrdLocation[];
  // Object keyed by id (real compiler format)
  if (typeof raw === 'object') {
    return Object.entries(raw as Record<string, unknown>).map(([id, val]) => {
      const obj = val as Record<string, unknown>;
      const loc: UrdLocation = {
        id,
        name: (obj.name as string) ?? id,
        description: (obj.description as string) ?? '',
        exits: normaliseExits(obj.exits),
      };
      if (obj.contains) loc.contains = obj.contains as string[];
      return loc;
    });
  }
  return [];
}

function normaliseExits(raw: unknown): UrdExit[] {
  if (!raw) return [];
  // Already an array (fixture format)
  if (Array.isArray(raw)) return raw as UrdExit[];
  // Object keyed by direction (real compiler format: { "north": { "to": "loc_id" } })
  if (typeof raw === 'object') {
    return Object.entries(raw as Record<string, unknown>).map(([direction, val]) => {
      const obj = val as Record<string, unknown>;
      return {
        direction,
        target: (obj.to as string) ?? '',
      };
    });
  }
  return [];
}

export class CompilerOutputCache {
  private cache = new Map<ChunkName, CacheEntry>();

  /**
   * Resolve a CompilerOutput into a ResolvedCompilerOutput.
   * Reuses cached values for chunks whose content hash hasn't changed.
   */
  resolve(output: CompilerOutput): ResolvedCompilerOutput {
    const resolved: Partial<ResolvedCompilerOutput> = {
      header: output.header,
    };

    for (const chunk of output.chunks) {
      const cached = this.cache.get(chunk.name);
      if (cached && cached.contentHash === chunk.contentHash) {
        this.assignChunk(resolved, chunk.name, cached.resolved);
      } else {
        const value = chunk.data;
        this.cache.set(chunk.name, {
          contentHash: chunk.contentHash,
          resolved: value,
        });
        this.assignChunk(resolved, chunk.name, value);
      }
    }

    // Stale-retention: for chunks missing from the current output, use the
    // previously cached value. This matches the playground pattern where
    // parsedWorld / definitionIndex survive failed compiles. Only fall back
    // to empty defaults when there is no prior cached value at all.
    const defaultFactSet: FactSet = { reads: [], writes: [], exits: [], jumps: [], choices: [], rules: [] };
    const defaultPropIdx: PropertyDependencyIndex = {
      properties: [],
      summary: { total_properties: 0, total_reads: 0, total_writes: 0, read_never_written: 0, written_never_read: 0 },
    };
    const defaultDefIdx: RichDefinitionIndex = { definitions: [], count: 0 };

    return {
      header: output.header,
      ast: (resolved.ast ?? this.staleOrDefault('ast', { nodes: [] })) as AST,
      symbolTable: (resolved.symbolTable ?? this.staleOrDefault('symbolTable', { entries: [] })) as SymbolTable,
      factSet: (resolved.factSet ?? this.staleOrDefault('factSet', defaultFactSet)) as FactSet,
      propertyDependencyIndex: (resolved.propertyDependencyIndex ?? this.staleOrDefault('propertyDependencyIndex', defaultPropIdx)) as PropertyDependencyIndex,
      definitionIndex: (resolved.definitionIndex ?? this.staleOrDefault('definitionIndex', defaultDefIdx)) as RichDefinitionIndex,
      urdJson: (resolved.urdJson ?? this.staleOrDefault('urdJson', { entities: [], locations: [] })) as UrdWorld,
      rawUrdJson: (resolved.rawUrdJson ?? this.staleRaw('urdJson')) as Record<string, unknown> | null,
      diagnostics: (resolved.diagnostics ?? this.staleOrDefault('diagnostics', [])) as Diagnostic[],
    };
  }

  /** Clear the cache. */
  clear(): void {
    this.cache.clear();
  }

  /** Returns the number of cached entries. */
  get size(): number {
    return this.cache.size;
  }

  /**
   * Return the previously cached value for a chunk, or the given default
   * if nothing has ever been cached for that chunk name.
   */
  private staleOrDefault(name: ChunkName, fallback: unknown): unknown {
    const cached = this.cache.get(name);
    if (cached) {
      // For urdJson, the cached raw value needs normalisation
      if (name === 'urdJson') return normaliseUrdJson(cached.resolved);
      return cached.resolved;
    }
    return fallback;
  }

  /** Return raw cached value for a chunk (no normalisation), or null. */
  private staleRaw(name: ChunkName): unknown {
    const cached = this.cache.get(name);
    return cached?.resolved ?? null;
  }

  private assignChunk(target: Partial<ResolvedCompilerOutput>, name: ChunkName, value: unknown): void {
    switch (name) {
      case 'ast':
        target.ast = value as AST;
        break;
      case 'symbolTable':
        target.symbolTable = value as SymbolTable;
        break;
      case 'factSet':
        target.factSet = value as FactSet;
        break;
      case 'propertyDependencyIndex':
        target.propertyDependencyIndex = value as PropertyDependencyIndex;
        break;
      case 'definitionIndex':
        target.definitionIndex = value as RichDefinitionIndex;
        break;
      case 'urdJson':
        target.urdJson = normaliseUrdJson(value);
        target.rawUrdJson = (value as Record<string, unknown>) ?? null;
        break;
      case 'diagnostics':
        target.diagnostics = value as Diagnostic[];
        break;
    }
  }
}
