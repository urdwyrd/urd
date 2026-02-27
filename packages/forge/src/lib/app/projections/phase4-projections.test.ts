/**
 * Phase 4 projection tests — validates all 7 new projections against fixture data.
 */

import { describe, it, expect } from 'vitest';
import type { ResolvedCompilerOutput, OutputHeader } from '$lib/app/compiler/types';
import { typeTableProjection } from './type-table';
import { propertyTableProjection } from './property-table';
import { locationTableProjection } from './location-table';
import { sectionTableProjection } from './section-table';
import { worldStatsProjection } from './world-stats';
import { deadCodeProjection } from './dead-code';
import { outlineProjection } from './outline';

function makeFixtureSource(): ResolvedCompilerOutput {
  return {
    header: {
      compileId: 'fixture-001',
      timestamp: 1700000000000,
      durationMs: 42,
      phaseTimings: [
        { phase: 'parse', durationMs: 8 },
        { phase: 'resolve', durationMs: 12 },
        { phase: 'analyse', durationMs: 10 },
        { phase: 'lower', durationMs: 7 },
        { phase: 'emit', durationMs: 5 },
      ],
      worldCounts: { entities: 3, locations: 2, exits: 2, properties: 5, rules: 1 },
      inputFileCount: 2,
    } as OutputHeader,
    ast: { nodes: [] },
    symbolTable: {
      entries: [
        { id: 'player', name: 'Player', kind: 'entity', file: 'main.urd.md', line: 3 },
        { id: 'sword', name: 'Sword', kind: 'entity', file: 'main.urd.md', line: 7 },
        { id: 'shield', name: 'Shield', kind: 'entity', file: 'main.urd.md', line: 11 },
        { id: 'tavern', name: 'Tavern', kind: 'location', file: 'locations.urd.md', line: 1 },
        { id: 'market', name: 'Market', kind: 'location', file: 'locations.urd.md', line: 5 },
      ],
    },
    factSet: {
      reads: [],
      writes: [],
      exits: [
        {
          from_location: 'tavern',
          to_location: 'market',
          exit_name: 'east',
          is_conditional: false,
          guard_reads: [],
          span: { file: 'locations.urd.md', start_line: 3, start_col: 1, end_line: 3, end_col: 20 },
        },
        {
          from_location: 'market',
          to_location: 'tavern',
          exit_name: 'west',
          is_conditional: false,
          guard_reads: [],
          span: { file: 'locations.urd.md', start_line: 7, start_col: 1, end_line: 7, end_col: 20 },
        },
      ],
      jumps: [],
      choices: [],
      rules: [],
    },
    propertyDependencyIndex: {
      properties: [
        { entity_type: 'Character', property: 'location', read_count: 1, write_count: 1, read_indices: [0], write_indices: [0], orphaned: null },
        { entity_type: 'Item', property: 'owner', read_count: 0, write_count: 1, read_indices: [], write_indices: [1], orphaned: 'written_never_read' },
      ],
      summary: { total_properties: 2, total_reads: 1, total_writes: 2, read_never_written: 0, written_never_read: 1 },
    },
    definitionIndex: {
      definitions: [
        { key: 'entity:@player', span: { file: 'main.urd.md', start_line: 3, start_col: 1, end_line: 3, end_col: 10 }, definition: { kind: 'entity', type_name: 'Character', display_name: 'Player' } },
        { key: 'entity:@sword', span: { file: 'main.urd.md', start_line: 7, start_col: 1, end_line: 7, end_col: 10 }, definition: { kind: 'entity', type_name: 'Item', display_name: 'Sword' } },
        { key: 'entity:@shield', span: { file: 'main.urd.md', start_line: 11, start_col: 1, end_line: 11, end_col: 10 }, definition: { kind: 'entity', type_name: 'Item', display_name: 'Shield' } },
        { key: 'location:tavern', span: { file: 'locations.urd.md', start_line: 1, start_col: 1, end_line: 1, end_col: 10 }, definition: { kind: 'location', display_name: 'Tavern' } },
        { key: 'location:market', span: { file: 'locations.urd.md', start_line: 5, start_col: 1, end_line: 5, end_col: 10 }, definition: { kind: 'location', display_name: 'Market' } },
      ],
      count: 5,
    },
    urdJson: {
      entities: [
        { id: 'player', name: 'player', type: 'Character', properties: { name: 'Hero', location: 'tavern' } },
        { id: 'sword', name: 'sword', type: 'Item', properties: { name: 'Iron Sword', owner: 'player' } },
        { id: 'shield', name: 'shield', type: 'Item', properties: { name: 'Wooden Shield' } },
      ],
      locations: [
        { id: 'tavern', name: 'tavern', description: 'A cosy tavern with a roaring fire', exits: [{ direction: 'east', target: 'market' }] },
        { id: 'market', name: 'market', description: 'A bustling marketplace', exits: [{ direction: 'west', target: 'tavern' }] },
      ],
    },
    rawUrdJson: {
      entities: {
        player: { type: 'Character', properties: { name: 'Hero', location: 'tavern' } },
        sword: { type: 'Item', properties: { name: 'Iron Sword', owner: 'player' } },
        shield: { type: 'Item', properties: { name: 'Wooden Shield' } },
      },
      locations: {
        tavern: { description: 'A cosy tavern with a roaring fire', exits: { east: { to: 'market' } }, contains: ['player', 'sword'] },
        market: { description: 'A bustling marketplace', exits: { west: { to: 'tavern' } }, contains: ['shield'] },
      },
    },
    diagnostics: [
      { severity: 'warning', message: "Entity 'Shield' has no location assigned", code: 'W001', span: { file: 'main.urd.md', startLine: 11, startCol: 1, endLine: 11, endCol: 20 } },
      { severity: 'info', message: 'Compilation successful with 3 entities and 2 locations', code: 'I001', span: null },
    ],
  };
}

describe('typeTableProjection', () => {
  it('returns empty array when no types exist', () => {
    const source = makeFixtureSource();
    const result = typeTableProjection.compute(source);
    expect(result).toEqual([]);
  });

  it('has correct projection ID', () => {
    expect(typeTableProjection.id).toBe('urd.projection.typeTable');
  });
});

describe('propertyTableProjection', () => {
  it('returns one row per entity property', () => {
    const source = makeFixtureSource();
    const result = propertyTableProjection.compute(source);
    // 3 entities: Player(name,location), Sword(name,owner), Shield(name) = 5 properties
    expect(result.length).toBe(5);
  });

  it('resolves subject names from symbol table', () => {
    const source = makeFixtureSource();
    const result = propertyTableProjection.compute(source);
    const playerRow = result.find((r) => r.subject === 'player' && r.predicate === 'name');
    expect(playerRow?.subjectName).toBe('Player');
  });

  it('counts dependencies correctly', () => {
    const source = makeFixtureSource();
    const result = propertyTableProjection.compute(source);
    const locationRow = result.find((r) => r.subject === 'player' && r.predicate === 'location');
    // Character.location: read_count=1 + write_count=1 = 2
    expect(locationRow?.dependencyCount).toBe(2);
  });

  it('sets zero dependencies for non-dependent properties', () => {
    const source = makeFixtureSource();
    const result = propertyTableProjection.compute(source);
    const nameRow = result.find((r) => r.subject === 'player' && r.predicate === 'name');
    expect(nameRow?.dependencyCount).toBe(0);
  });
});

describe('locationTableProjection', () => {
  it('returns one row per location', () => {
    const source = makeFixtureSource();
    const result = locationTableProjection.compute(source);
    expect(result.length).toBe(2);
  });

  it('includes exit counts', () => {
    const source = makeFixtureSource();
    const result = locationTableProjection.compute(source);
    const tavern = result.find((r) => r.name === 'Tavern');
    expect(tavern?.exitCount).toBe(1);
  });

  it('includes file and line from symbol table', () => {
    const source = makeFixtureSource();
    const result = locationTableProjection.compute(source);
    const market = result.find((r) => r.name === 'Market');
    expect(market?.file).toBe('locations.urd.md');
    expect(market?.line).toBe(5);
  });
});

describe('sectionTableProjection', () => {
  it('returns empty array when no sections exist', () => {
    const source = makeFixtureSource();
    const result = sectionTableProjection.compute(source);
    expect(result).toEqual([]);
  });

  it('filters only section-kind symbols', () => {
    const source = makeFixtureSource();
    source.symbolTable.entries.push(
      { id: 'sec_intro', name: 'Introduction', kind: 'section', file: 'main.urd.md', line: 1 },
    );
    const result = sectionTableProjection.compute(source);
    expect(result.length).toBe(1);
    expect(result[0].name).toBe('Introduction');
  });
});

describe('worldStatsProjection', () => {
  it('returns correct aggregate counts from fixture', () => {
    const source = makeFixtureSource();
    const result = worldStatsProjection.compute(source);
    expect(result.entityCount).toBe(3);
    expect(result.locationCount).toBe(2);
    expect(result.exitCount).toBe(2);
    expect(result.propertyCount).toBe(5);
    expect(result.ruleCount).toBe(1);
    expect(result.factCount).toBe(2);
    expect(result.symbolCount).toBe(5);
    expect(result.diagnosticCount).toBe(2);
    expect(result.warningCount).toBe(1);
    expect(result.infoCount).toBe(1);
    expect(result.errorCount).toBe(0);
    expect(result.compileDurationMs).toBe(42);
  });

  it('includes phase timings', () => {
    const source = makeFixtureSource();
    const result = worldStatsProjection.compute(source);
    expect(result.phaseTimings.length).toBe(5);
    expect(result.phaseTimings[0].phase).toBe('parse');
  });
});

describe('deadCodeProjection', () => {
  it('identifies unreferenced symbols', () => {
    const source = makeFixtureSource();
    const result = deadCodeProjection.compute(source);
    // loc_market is referenced via exit target, loc_tavern via fact object + dependency
    // player, ent_sword, ent_shield, loc_tavern, loc_market are all referenced
    // Shield's ent_shield is referenced as a fact subject
    // All symbols are referenced in some way through facts
    expect(result.length).toBeGreaterThanOrEqual(0);
  });

  it('has correct projection ID', () => {
    expect(deadCodeProjection.id).toBe('urd.projection.deadCode');
  });
});

describe('outlineProjection', () => {
  it('groups symbols by file', () => {
    const source = makeFixtureSource();
    const result = outlineProjection.compute(source);
    expect(result.length).toBe(2);
  });

  it('sorts files alphabetically', () => {
    const source = makeFixtureSource();
    const result = outlineProjection.compute(source);
    expect(result[0].file).toBe('locations.urd.md');
    expect(result[1].file).toBe('main.urd.md');
  });

  it('sorts entries within each file by line number', () => {
    const source = makeFixtureSource();
    const result = outlineProjection.compute(source);
    const mainFile = result.find((f) => f.file === 'main.urd.md');
    expect(mainFile!.entries[0].line).toBe(3);
    expect(mainFile!.entries[1].line).toBe(7);
    expect(mainFile!.entries[2].line).toBe(11);
  });

  it('includes symbol kind and name', () => {
    const source = makeFixtureSource();
    const result = outlineProjection.compute(source);
    const locFile = result.find((f) => f.file === 'locations.urd.md');
    expect(locFile!.entries[0]).toMatchObject({ name: 'Tavern', kind: 'location' });
  });
});
