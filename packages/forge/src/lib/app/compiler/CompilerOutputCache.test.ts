import { describe, it, expect, beforeEach } from 'vitest';
import { CompilerOutputCache } from './CompilerOutputCache';
import type { CompilerOutput, Chunk, OutputHeader } from './types';

function makeHeader(overrides: Partial<OutputHeader> = {}): OutputHeader {
  return {
    compileId: 'test-001',
    timestamp: Date.now(),
    durationMs: 10,
    phaseTimings: [],
    worldCounts: { entities: 0, locations: 0, exits: 0, properties: 0, rules: 0 },
    inputFileCount: 1,
    ...overrides,
  };
}

function makeChunk(name: string, data: unknown, hash: string): Chunk {
  return { name: name as Chunk['name'], data, contentHash: hash };
}

describe('CompilerOutputCache', () => {
  let cache: CompilerOutputCache;

  beforeEach(() => {
    cache = new CompilerOutputCache();
  });

  it('resolves all chunk types from output', () => {
    const output: CompilerOutput = {
      header: makeHeader(),
      chunks: [
        makeChunk('ast', { nodes: [{ kind: 'world' }] }, 'hash1'),
        makeChunk('symbolTable', { entries: [{ id: 'e1', name: 'Foo', kind: 'entity', file: 'a.urd.md', line: 1 }] }, 'hash2'),
        makeChunk('factSet', { facts: [] }, 'hash3'),
        makeChunk('propertyDependencyIndex', { dependencies: [] }, 'hash4'),
        makeChunk('urdJson', { entities: [], locations: [] }, 'hash5'),
        makeChunk('diagnostics', [], 'hash6'),
      ],
    };

    const resolved = cache.resolve(output);
    expect(resolved.header).toBe(output.header);
    expect(resolved.ast.nodes).toHaveLength(1);
    expect(resolved.symbolTable.entries).toHaveLength(1);
    expect(resolved.factSet.facts).toHaveLength(0);
    expect(resolved.diagnostics).toHaveLength(0);
  });

  it('uses cached values when hashes match', () => {
    const data1 = { nodes: [{ kind: 'world' }] };
    const output1: CompilerOutput = {
      header: makeHeader({ compileId: 'c1' }),
      chunks: [makeChunk('ast', data1, 'same-hash')],
    };

    const resolved1 = cache.resolve(output1);

    // Second compile with same hash but different object reference
    const data2 = { nodes: [{ kind: 'different' }] };
    const output2: CompilerOutput = {
      header: makeHeader({ compileId: 'c2' }),
      chunks: [makeChunk('ast', data2, 'same-hash')],
    };

    const resolved2 = cache.resolve(output2);

    // Should get cached value (data1), not data2
    expect(resolved2.ast).toBe(resolved1.ast);
    expect(resolved2.ast.nodes[0]).toEqual({ kind: 'world' });
  });

  it('updates cached values when hashes differ', () => {
    const output1: CompilerOutput = {
      header: makeHeader(),
      chunks: [makeChunk('ast', { nodes: [{ kind: 'v1' }] }, 'hash-v1')],
    };
    cache.resolve(output1);

    const output2: CompilerOutput = {
      header: makeHeader(),
      chunks: [makeChunk('ast', { nodes: [{ kind: 'v2' }] }, 'hash-v2')],
    };
    const resolved2 = cache.resolve(output2);

    expect(resolved2.ast.nodes[0]).toEqual({ kind: 'v2' });
  });

  it('provides defaults for missing chunks', () => {
    const output: CompilerOutput = {
      header: makeHeader(),
      chunks: [], // no chunks at all
    };

    const resolved = cache.resolve(output);
    expect(resolved.ast).toEqual({ nodes: [] });
    expect(resolved.symbolTable).toEqual({ entries: [] });
    expect(resolved.factSet).toEqual({ facts: [] });
    expect(resolved.propertyDependencyIndex).toEqual({ dependencies: [] });
    expect(resolved.urdJson).toEqual({ entities: [], locations: [] });
    expect(resolved.diagnostics).toEqual([]);
  });

  it('clear() empties the cache', () => {
    const output: CompilerOutput = {
      header: makeHeader(),
      chunks: [makeChunk('ast', { nodes: [] }, 'hash1')],
    };
    cache.resolve(output);
    expect(cache.size).toBe(1);

    cache.clear();
    expect(cache.size).toBe(0);
  });
});
