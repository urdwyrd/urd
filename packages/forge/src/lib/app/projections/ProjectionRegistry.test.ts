import { describe, it, expect, beforeEach } from 'vitest';
import { ProjectionRegistry, type ProjectionDefinition } from './ProjectionRegistry';
import type { ResolvedCompilerOutput, OutputHeader } from '$lib/app/compiler/types';

function makeSource(overrides: Partial<ResolvedCompilerOutput> = {}): ResolvedCompilerOutput {
  return {
    header: {
      compileId: 'test',
      timestamp: Date.now(),
      durationMs: 10,
      phaseTimings: [],
      worldCounts: { entities: 0, locations: 0, exits: 0, properties: 0, rules: 0 },
      inputFileCount: 1,
    } as OutputHeader,
    ast: { nodes: [] },
    symbolTable: { entries: [] },
    factSet: { facts: [] },
    propertyDependencyIndex: { dependencies: [] },
    urdJson: { entities: [], locations: [] },
    diagnostics: [],
    ...overrides,
  };
}

describe('ProjectionRegistry', () => {
  let registry: ProjectionRegistry;

  beforeEach(() => {
    registry = new ProjectionRegistry();
  });

  it('get() returns null when no source is loaded', () => {
    const projection: ProjectionDefinition<number> = {
      id: 'test.count',
      depends: ['urdJson'],
      compute: (src) => src.urdJson.entities.length,
    };
    registry.register(projection);

    expect(registry.get('test.count')).toBeNull();
  });

  it('get() returns null for unregistered projections', () => {
    expect(registry.get('nope')).toBeNull();
  });

  it('computes projection from source', () => {
    const projection: ProjectionDefinition<number> = {
      id: 'test.entityCount',
      depends: ['urdJson'],
      compute: (src) => src.urdJson.entities.length,
    };
    registry.register(projection);

    const source = makeSource({
      urdJson: {
        entities: [
          { id: 'e1', name: 'A', properties: {} },
          { id: 'e2', name: 'B', properties: {} },
        ],
        locations: [],
      },
    });
    registry.updateSource(source, { urdJson: 'hash-1' });

    expect(registry.get<number>('test.entityCount')).toBe(2);
  });

  it('caches result when dependency hashes are unchanged', () => {
    let computeCount = 0;
    const projection: ProjectionDefinition<string[]> = {
      id: 'test.names',
      depends: ['urdJson'],
      compute: (src) => {
        computeCount++;
        return src.urdJson.entities.map((e) => e.name);
      },
    };
    registry.register(projection);

    const source = makeSource();
    registry.updateSource(source, { urdJson: 'hash-same' });

    registry.get('test.names');
    registry.get('test.names');

    expect(computeCount).toBe(1);
  });

  it('recomputes when dependency hashes change', () => {
    let computeCount = 0;
    const projection: ProjectionDefinition<number> = {
      id: 'test.counter',
      depends: ['urdJson'],
      compute: () => ++computeCount,
    };
    registry.register(projection);

    registry.updateSource(makeSource(), { urdJson: 'hash-v1' });
    expect(registry.get<number>('test.counter')).toBe(1);

    registry.updateSource(makeSource(), { urdJson: 'hash-v2' });
    expect(registry.get<number>('test.counter')).toBe(2);
  });

  it('handles multiple dependency hashes', () => {
    let computeCount = 0;
    const projection: ProjectionDefinition<number> = {
      id: 'test.multi',
      depends: ['symbolTable', 'urdJson'],
      compute: () => ++computeCount,
    };
    registry.register(projection);

    registry.updateSource(makeSource(), { symbolTable: 'st-1', urdJson: 'uj-1' });
    expect(registry.get<number>('test.multi')).toBe(1);

    // Only one dependency changes
    registry.updateSource(makeSource(), { symbolTable: 'st-2', urdJson: 'uj-1' });
    expect(registry.get<number>('test.multi')).toBe(2);
  });

  it('clear() resets cached results', () => {
    const projection: ProjectionDefinition<number> = {
      id: 'test.val',
      depends: ['urdJson'],
      compute: () => 42,
    };
    registry.register(projection);

    registry.updateSource(makeSource(), { urdJson: 'h1' });
    expect(registry.get<number>('test.val')).toBe(42);

    registry.clear();
    expect(registry.get<number>('test.val')).toBeNull();
  });

  it('list() returns all projection IDs', () => {
    registry.register({ id: 'a', depends: [], compute: () => null });
    registry.register({ id: 'b', depends: [], compute: () => null });
    expect(registry.list().sort()).toEqual(['a', 'b']);
  });
});
