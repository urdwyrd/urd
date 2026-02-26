import { describe, it, expect, beforeEach } from 'vitest';
import { MockCompilerService } from './MockCompilerService';

describe('MockCompilerService', () => {
  let compiler: MockCompilerService;

  beforeEach(() => {
    compiler = new MockCompilerService();
    compiler.delayMs = 0; // no delay in tests
  });

  it('returns CompilerOutput with all chunk types', async () => {
    const output = await compiler.compile({ '/main.urd.md': '# World: Test' });

    expect(output.header).toBeDefined();
    expect(output.header.compileId).toMatch(/^mock-/);
    expect(output.header.inputFileCount).toBe(1);
    expect(output.chunks).toHaveLength(6);

    const chunkNames = output.chunks.map((c) => c.name);
    expect(chunkNames).toContain('ast');
    expect(chunkNames).toContain('symbolTable');
    expect(chunkNames).toContain('factSet');
    expect(chunkNames).toContain('propertyDependencyIndex');
    expect(chunkNames).toContain('urdJson');
    expect(chunkNames).toContain('diagnostics');
  });

  it('chunks have content hashes', async () => {
    const output = await compiler.compile({ '/main.urd.md': 'content' });

    for (const chunk of output.chunks) {
      expect(chunk.contentHash).toBeTruthy();
      expect(typeof chunk.contentHash).toBe('string');
    }
  });

  it('returns same hashes for same input', async () => {
    const buffers = { '/main.urd.md': '# World: Test' };

    const output1 = await compiler.compile(buffers);
    const output2 = await compiler.compile(buffers);

    expect(output1.header.compileId).not.toBe(output2.header.compileId);

    // Hashes should be the same since input didn't change
    for (let i = 0; i < output1.chunks.length; i++) {
      expect(output1.chunks[i].contentHash).toBe(output2.chunks[i].contentHash);
    }
  });

  it('returns different hashes for different input', async () => {
    const output1 = await compiler.compile({ '/a.urd.md': 'version 1' });
    const output2 = await compiler.compile({ '/a.urd.md': 'version 2' });

    // At least some hashes should differ
    const hashesChanged = output1.chunks.some(
      (c, i) => c.contentHash !== output2.chunks[i].contentHash
    );
    expect(hashesChanged).toBe(true);
  });

  it('header reflects input file count', async () => {
    const output = await compiler.compile({
      '/a.urd.md': 'a',
      '/b.urd.md': 'b',
      '/c.urd.md': 'c',
    });
    expect(output.header.inputFileCount).toBe(3);
  });

  it('world counts come from fixture data', async () => {
    const output = await compiler.compile({ '/main.urd.md': 'test' });
    expect(output.header.worldCounts.entities).toBe(3);
    expect(output.header.worldCounts.locations).toBe(2);
    expect(output.header.worldCounts.exits).toBe(2);
  });
});
