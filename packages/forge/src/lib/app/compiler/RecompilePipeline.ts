/**
 * Recompile pipeline — orchestrates buffer changes through compilation
 * to projection updates.
 *
 * Flow: BufferMap change → debounce → compiler.started → CompilerService.compile
 * → CompilerOutputCache.resolve → projectionRegistry.updateSource → compiler.completed
 */

import { bus } from '$lib/framework/bus/MessageBus';
import { BufferMap } from './BufferMap';
import { CompilerOutputCache } from './CompilerOutputCache';
import type { CompilerService, CompilerOutput } from './types';
import { ProjectionRegistry } from '$lib/app/projections/ProjectionRegistry';

export class RecompilePipeline {
  private debounceTimer: ReturnType<typeof setTimeout> | null = null;
  private compileCounter = 0;
  private isCompiling = false;
  private pendingRecompile = false;
  private unsubBuffer: (() => void) | null = null;
  private unsubActiveFile: (() => void) | null = null;
  private activeFile: string | null = null;

  constructor(
    private bufferMap: BufferMap,
    private compiler: CompilerService,
    private cache: CompilerOutputCache,
    private projectionRegistry: ProjectionRegistry,
    private debounceMs: number = 300,
  ) {}

  /** Start listening for buffer changes and active file switches. */
  start(): void {
    this.unsubBuffer = this.bufferMap.subscribe(() => {
      this.scheduleCompile();
    });

    // Recompile when the active editor file changes — the entry point shifts
    if (bus.hasChannel('editor.activeFile')) {
      this.unsubActiveFile = bus.subscribe('editor.activeFile', (payload) => {
        const { path } = payload as { path: string | null };
        if (path && path !== this.activeFile && path.endsWith('.urd.md')) {
          this.activeFile = path;
          this.scheduleCompile();
        }
      });
    }
  }

  /** Stop listening and cancel any pending compile. */
  stop(): void {
    this.unsubBuffer?.();
    this.unsubBuffer = null;
    this.unsubActiveFile?.();
    this.unsubActiveFile = null;
    this.cancelPending();
  }

  /** Trigger an immediate compile (bypassing debounce). */
  async compileNow(): Promise<void> {
    this.cancelPending();
    await this.runCompile();
  }

  /** Schedule a debounced compile. */
  private scheduleCompile(): void {
    if (this.debounceTimer) {
      clearTimeout(this.debounceTimer);
    }
    this.debounceTimer = setTimeout(() => {
      this.debounceTimer = null;
      this.runCompile();
    }, this.debounceMs);
  }

  private cancelPending(): void {
    if (this.debounceTimer) {
      clearTimeout(this.debounceTimer);
      this.debounceTimer = null;
    }
  }

  private async runCompile(): Promise<void> {
    // If already compiling, mark pending to recompile after
    if (this.isCompiling) {
      this.pendingRecompile = true;
      return;
    }

    this.isCompiling = true;
    const compileId = `compile-${++this.compileCounter}`;
    const buffers = this.bufferMap.getAll();
    const fileCount = Object.keys(buffers).length;

    if (fileCount === 0) {
      this.isCompiling = false;
      return;
    }

    // Signal: compiler started
    if (bus.hasChannel('compiler.started')) {
      bus.publish('compiler.started', {
        compileId,
        timestamp: Date.now(),
        inputFileCount: fileCount,
      });
    }

    try {
      const output: CompilerOutput = await this.compiler.compile(buffers, this.activeFile ?? undefined);

      // Resolve through cache (de-duplicate unchanged chunks)
      const resolved = this.cache.resolve(output);

      // Build chunk hash map
      const chunkHashes: Record<string, string> = {};
      for (const chunk of output.chunks) {
        chunkHashes[chunk.name] = chunk.contentHash;
      }

      if (import.meta.env?.DEV) {
        console.warn('[RecompilePipeline] compile done, chunks:', Object.keys(chunkHashes),
          'definitionIndex defs:', resolved.definitionIndex?.definitions?.length ?? 0,
          'rawUrdJson:', resolved.rawUrdJson != null,
          'factSet reads:', resolved.factSet?.reads?.length ?? 0,
        );
      }

      // Update projections
      this.projectionRegistry.updateSource(resolved, chunkHashes);

      // Signal: compiler completed
      if (bus.hasChannel('compiler.completed')) {
        bus.publish('compiler.completed', {
          compileId,
          durationMs: output.header.durationMs,
          chunkHashes,
          worldCounts: {
            entities: output.header.worldCounts.entities,
            locations: output.header.worldCounts.locations,
            exits: output.header.worldCounts.exits,
          },
        });
      }
    } catch (err) {
      console.error('Compile failed:', err);
      if (bus.hasChannel('compiler.error')) {
        bus.publish('compiler.error', {
          compileId,
          error: err instanceof Error ? err.message : String(err),
        });
      }
    } finally {
      this.isCompiling = false;

      // If a recompile was requested during compilation, run it now
      if (this.pendingRecompile) {
        this.pendingRecompile = false;
        this.scheduleCompile();
      }
    }
  }
}
