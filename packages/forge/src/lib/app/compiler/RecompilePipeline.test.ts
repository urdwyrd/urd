import { describe, it, expect, beforeEach, vi } from 'vitest';
import { RecompilePipeline } from './RecompilePipeline';
import { BufferMap } from './BufferMap';
import { CompilerOutputCache } from './CompilerOutputCache';
import { MockCompilerService } from './MockCompilerService';
import { ProjectionRegistry } from '$lib/app/projections/ProjectionRegistry';

describe('RecompilePipeline', () => {
  let bm: BufferMap;
  let compiler: MockCompilerService;
  let cache: CompilerOutputCache;
  let projections: ProjectionRegistry;
  let pipeline: RecompilePipeline;

  beforeEach(() => {
    bm = new BufferMap();
    compiler = new MockCompilerService();
    compiler.delayMs = 0;
    cache = new CompilerOutputCache();
    projections = new ProjectionRegistry();
    pipeline = new RecompilePipeline(bm, compiler, cache, projections, 0);
  });

  it('compileNow() runs compilation and updates projections', async () => {
    projections.register({
      id: 'test.entities',
      depends: ['urdJson'],
      compute: (src) => src.urdJson.entities.length,
    });

    bm.load('/main.urd.md', '# World: Test');

    await pipeline.compileNow();

    const entityCount = projections.get<number>('test.entities');
    expect(entityCount).not.toBeNull();
    // MockCompiler returns fixture data with 3 entities
    expect(entityCount).toBe(3);
  });

  it('compileNow() skips compilation when buffer is empty', async () => {
    const compileSpy = vi.spyOn(compiler, 'compile');

    await pipeline.compileNow();

    expect(compileSpy).not.toHaveBeenCalled();
  });

  it('start() subscribes to buffer changes and triggers compile', async () => {
    bm.load('/a.urd.md', 'initial');
    pipeline.start();

    // Trigger a buffer change
    bm.set('/a.urd.md', 'changed');

    // Wait for debounce (0ms) + microtask/setTimeout
    await new Promise((resolve) => setTimeout(resolve, 50));

    pipeline.stop();

    // Cache should be populated from the compile
    expect(cache.size).toBeGreaterThan(0);
  });

  it('stop() cancels pending compilation', async () => {
    const compileSpy = vi.spyOn(compiler, 'compile');

    bm.load('/a.urd.md', 'initial');

    // Use a longer debounce so we can stop before it fires
    pipeline = new RecompilePipeline(bm, compiler, cache, projections, 500);
    pipeline.start();
    bm.set('/a.urd.md', 'changed');

    // Stop immediately before debounce fires
    pipeline.stop();

    await new Promise((resolve) => setTimeout(resolve, 600));

    expect(compileSpy).not.toHaveBeenCalled();
  });

  it('does not crash on compiler errors', async () => {
    compiler.compile = async () => {
      throw new Error('Test compile error');
    };

    bm.load('/a.urd.md', 'content');

    await expect(pipeline.compileNow()).resolves.toBeUndefined();
  });

  it('multiple compileNow calls work sequentially', async () => {
    bm.load('/a.urd.md', 'v1');
    await pipeline.compileNow();

    bm.set('/a.urd.md', 'v2');
    await pipeline.compileNow();

    // Both compiles should have succeeded
    expect(cache.size).toBeGreaterThan(0);
  });
});
