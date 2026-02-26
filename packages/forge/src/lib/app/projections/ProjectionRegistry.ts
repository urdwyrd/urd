/**
 * Projection registry — memoised derived views over compiler output.
 *
 * Projections are lazy: `updateSource()` stores the new output but does NOT
 * recompute. Projections recompute on `get()` only when dependency hashes change.
 * In dev mode, projection results are Object.frozen to catch mutation.
 */

import type {
  ResolvedCompilerOutput,
  ChunkName,
} from '$lib/app/compiler/types';
import { bus } from '$lib/framework/bus/MessageBus';

export interface ProjectionDefinition<T = unknown> {
  id: string;
  depends: ChunkName[];
  compute: (source: ResolvedCompilerOutput) => T;
}

interface ProjectionEntry<T = unknown> {
  definition: ProjectionDefinition<T>;
  cachedResult: T | null;
  cachedDependencyHash: string | null;
}

const IS_DEV = typeof import.meta !== 'undefined' && import.meta.env?.DEV;

export class ProjectionRegistry {
  private projections = new Map<string, ProjectionEntry>();
  private source: ResolvedCompilerOutput | null = null;
  private chunkHashes = new Map<ChunkName, string>();

  /** Register a projection definition. */
  register<T>(definition: ProjectionDefinition<T>): void {
    this.projections.set(definition.id, {
      definition: definition as ProjectionDefinition,
      cachedResult: null,
      cachedDependencyHash: null,
    });
  }

  /**
   * Update the compiler output source. This stores the new output and
   * chunk hashes but does NOT recompute any projections. Views call get()
   * to trigger lazy recomputation.
   */
  updateSource(output: ResolvedCompilerOutput, hashes: Record<string, string>): void {
    this.source = output;
    this.chunkHashes.clear();
    for (const [name, hash] of Object.entries(hashes)) {
      this.chunkHashes.set(name as ChunkName, hash);
    }

    if (bus.hasChannel('projection.updated')) {
      bus.publish('projection.updated', {
        projectionIds: Array.from(this.projections.keys()),
      });
    }
  }

  /**
   * Get a projection's result. Recomputes if dependency hashes have changed.
   * Returns null if no source has been loaded yet.
   */
  get<T>(projectionId: string): T | null {
    const entry = this.projections.get(projectionId);
    if (!entry || !this.source) return null;

    const depHash = this.computeDependencyHash(entry.definition.depends);

    if (entry.cachedDependencyHash === depHash && entry.cachedResult !== null) {
      return entry.cachedResult as T;
    }

    // Recompute
    const result = entry.definition.compute(this.source);
    const frozen = IS_DEV ? deepFreeze(result) : result;
    entry.cachedResult = frozen;
    entry.cachedDependencyHash = depHash;
    return frozen as T;
  }

  /** Returns all registered projection IDs. */
  list(): string[] {
    return Array.from(this.projections.keys());
  }

  /** Clear all cached results (e.g., on project close). */
  clear(): void {
    this.source = null;
    this.chunkHashes.clear();
    for (const entry of this.projections.values()) {
      entry.cachedResult = null;
      entry.cachedDependencyHash = null;
    }
  }

  private computeDependencyHash(depends: ChunkName[]): string {
    return depends
      .map((name) => this.chunkHashes.get(name) ?? 'missing')
      .join(':');
  }
}

function deepFreeze<T>(obj: T): T {
  if (obj === null || typeof obj !== 'object') return obj;
  Object.freeze(obj);
  for (const value of Object.values(obj as Record<string, unknown>)) {
    if (typeof value === 'object' && value !== null && !Object.isFrozen(value)) {
      deepFreeze(value);
    }
  }
  return obj;
}

/** Singleton projection registry. */
export const projectionRegistry = new ProjectionRegistry();
