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
        makeChunk('factSet', { reads: [], writes: [], exits: [], jumps: [], choices: [], rules: [] }, 'hash3'),
        makeChunk('propertyDependencyIndex', { properties: [], summary: { total_properties: 0, total_reads: 0, total_writes: 0, read_never_written: 0, written_never_read: 0 } }, 'hash4'),
        makeChunk('definitionIndex', { definitions: [], count: 0 }, 'hash4b'),
        makeChunk('urdJson', { entities: [], locations: [] }, 'hash5'),
        makeChunk('diagnostics', [], 'hash6'),
      ],
    };

    const resolved = cache.resolve(output);
    expect(resolved.header).toBe(output.header);
    expect(resolved.ast.nodes).toHaveLength(1);
    expect(resolved.symbolTable.entries).toHaveLength(1);
    expect(resolved.factSet.reads).toHaveLength(0);
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
    expect(resolved.factSet).toEqual({ reads: [], writes: [], exits: [], jumps: [], choices: [], rules: [] });
    expect(resolved.propertyDependencyIndex).toEqual({ properties: [], summary: { total_properties: 0, total_reads: 0, total_writes: 0, read_never_written: 0, written_never_read: 0 } });
    expect(resolved.definitionIndex).toEqual({ definitions: [], count: 0 });
    expect(resolved.urdJson).toEqual({ entities: [], locations: [] });
    expect(resolved.diagnostics).toEqual([]);
  });

  it('retains stale data when chunks are missing (stale-retention)', () => {
    // First compile succeeds — all chunks present
    const output1: CompilerOutput = {
      header: makeHeader({ compileId: 'c1' }),
      chunks: [
        makeChunk('ast', { nodes: [{ kind: 'world' }] }, 'hash-ast'),
        makeChunk('symbolTable', { entries: [{ id: 'e1', name: 'Foo', kind: 'entity', file: 'a.urd.md', line: 1 }] }, 'hash-st'),
        makeChunk('factSet', { reads: [{ entity: 'e1', property: 'name' }], writes: [], exits: [], jumps: [], choices: [], rules: [] }, 'hash-fs'),
        makeChunk('definitionIndex', { definitions: [{ key: 'entity:@Foo' }], count: 1 }, 'hash-di'),
        makeChunk('urdJson', { entities: { player: { type: 'Character' } }, locations: {} }, 'hash-uj'),
        makeChunk('diagnostics', [], 'hash-dx'),
      ],
    };
    const resolved1 = cache.resolve(output1);
    expect(resolved1.factSet.reads).toHaveLength(1);
    expect(resolved1.definitionIndex.definitions).toHaveLength(1);
    expect(resolved1.urdJson.entities).toHaveLength(1);

    // Second compile fails — only ast, symbolTable, diagnostics present
    // (factSet, definitionIndex, urdJson omitted by bridge)
    const output2: CompilerOutput = {
      header: makeHeader({ compileId: 'c2' }),
      chunks: [
        makeChunk('ast', { nodes: [] }, 'hash-ast-2'),
        makeChunk('symbolTable', { entries: [] }, 'hash-st-2'),
        makeChunk('diagnostics', [{ severity: 'error', message: 'syntax error', code: 'E001' }], 'hash-dx-2'),
      ],
    };
    const resolved2 = cache.resolve(output2);

    // Stale retention: factSet, definitionIndex, urdJson should survive
    expect(resolved2.factSet.reads).toHaveLength(1);
    expect(resolved2.definitionIndex.definitions).toHaveLength(1);
    expect(resolved2.urdJson.entities).toHaveLength(1);

    // But ast, symbolTable, diagnostics should be updated
    expect(resolved2.ast.nodes).toHaveLength(0);
    expect(resolved2.symbolTable.entries).toHaveLength(0);
    expect(resolved2.diagnostics).toHaveLength(1);
  });

  it('normalises urdJson from object-keyed format (real compiler)', () => {
    // The real compiler emits entities/locations as { "id": { ... } } objects
    const output: CompilerOutput = {
      header: makeHeader(),
      chunks: [
        makeChunk('urdJson', {
          world: { name: 'Demo' },
          entities: {
            player: { type: 'Character', properties: { name: 'Hero' } },
            sword: { type: 'Weapon', properties: { name: 'Iron Sword' } },
          },
          locations: {
            tavern: { description: 'A cosy tavern', exits: { east: { to: 'market' } } },
            market: { description: 'A bustling market', exits: { west: { to: 'tavern' } } },
          },
        }, 'hash-obj'),
      ],
    };

    const resolved = cache.resolve(output);
    // entities should be normalised to an array
    expect(Array.isArray(resolved.urdJson.entities)).toBe(true);
    expect(resolved.urdJson.entities).toHaveLength(2);
    expect(resolved.urdJson.entities[0].id).toBe('player');
    expect(resolved.urdJson.entities[0].properties).toEqual({ name: 'Hero' });

    // locations should be normalised to an array
    expect(Array.isArray(resolved.urdJson.locations)).toBe(true);
    expect(resolved.urdJson.locations).toHaveLength(2);
    expect(resolved.urdJson.locations[0].id).toBe('tavern');
    expect(resolved.urdJson.locations[0].description).toBe('A cosy tavern');

    // exits should be normalised to an array
    expect(Array.isArray(resolved.urdJson.locations[0].exits)).toBe(true);
    expect(resolved.urdJson.locations[0].exits[0].direction).toBe('east');
    expect(resolved.urdJson.locations[0].exits[0].target).toBe('market');
  });

  it('stores rawUrdJson alongside normalised urdJson', () => {
    const rawData = {
      entities: { player: { type: 'Character', properties: { name: 'Hero' } } },
      locations: { tavern: { description: 'A tavern', exits: { east: { to: 'market' } } } },
    };
    const output: CompilerOutput = {
      header: makeHeader(),
      chunks: [makeChunk('urdJson', rawData, 'hash-raw')],
    };

    const resolved = cache.resolve(output);
    // normalised: arrays
    expect(resolved.urdJson.entities).toHaveLength(1);
    expect(resolved.urdJson.entities[0].id).toBe('player');
    // raw: object-keyed, as-is
    expect(resolved.rawUrdJson).toBeDefined();
    expect((resolved.rawUrdJson as Record<string, unknown>).entities).toEqual(rawData.entities);
    expect((resolved.rawUrdJson as Record<string, unknown>).locations).toEqual(rawData.locations);
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
