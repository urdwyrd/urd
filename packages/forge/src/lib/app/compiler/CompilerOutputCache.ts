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
  UrdWorld,
  Diagnostic,
} from './types';

interface CacheEntry {
  contentHash: string;
  resolved: unknown;
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

    // Fill defaults for missing chunks
    return {
      header: output.header,
      ast: (resolved.ast ?? { nodes: [] }) as AST,
      symbolTable: (resolved.symbolTable ?? { entries: [] }) as SymbolTable,
      factSet: (resolved.factSet ?? { facts: [] }) as FactSet,
      propertyDependencyIndex: (resolved.propertyDependencyIndex ?? { dependencies: [] }) as PropertyDependencyIndex,
      urdJson: (resolved.urdJson ?? { entities: [], locations: [] }) as UrdWorld,
      diagnostics: (resolved.diagnostics ?? []) as Diagnostic[],
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
      case 'urdJson':
        target.urdJson = value as UrdWorld;
        break;
      case 'diagnostics':
        target.diagnostics = value as Diagnostic[];
        break;
    }
  }
}
