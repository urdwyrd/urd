/**
 * Stress fixture generator — creates large CompilerOutput fixtures
 * for performance testing of Forge views and projections.
 *
 * Uses a deterministic seeded PRNG for reproducible output.
 * Run: npx tsx scripts/generate-stress-fixtures.ts
 */

import { writeFileSync, mkdirSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const fixturesDir = join(__dirname, '..', 'fixtures');
const publicFixturesDir = join(__dirname, '..', 'public', 'fixtures');

// ===== Seeded PRNG (Mulberry32) =====

function mulberry32(seed: number) {
  return () => {
    let t = seed += 0x6D2B79F5;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

const random = mulberry32(42);

function randomInt(min: number, max: number): number {
  return Math.floor(random() * (max - min + 1)) + min;
}

function randomPick<T>(arr: T[]): T {
  return arr[Math.floor(random() * arr.length)];
}

function randomWord(): string {
  const syllables = ['ath', 'bor', 'cel', 'dun', 'eld', 'fen', 'gal', 'har', 'ith', 'jar',
    'kel', 'lor', 'mar', 'nor', 'oth', 'pel', 'qar', 'ren', 'sol', 'tar',
    'und', 'val', 'wen', 'xar', 'yol', 'zel'];
  const count = randomInt(1, 3);
  let word = '';
  for (let i = 0; i < count; i++) word += randomPick(syllables);
  return word.charAt(0).toUpperCase() + word.slice(1);
}

function simpleHash(input: string): string {
  let hash = 0;
  for (let i = 0; i < input.length; i++) {
    const char = input.charCodeAt(i);
    hash = ((hash << 5) - hash) + char;
    hash |= 0;
  }
  return Math.abs(hash).toString(16).padStart(8, '0');
}

// ===== File names =====

function generateFiles(count: number): string[] {
  const files: string[] = [];
  for (let i = 0; i < count; i++) {
    files.push(`${randomWord().toLowerCase()}.urd.md`);
  }
  return [...new Set(files)]; // deduplicate
}

// ===== Types =====

interface StressConfig {
  name: string;
  entityCount: number;
  typeCount: number;
  locationCount: number;
  propertyCount: number;
  ruleCount: number;
  readCount: number;
  writeCount: number;
  exitCount: number;
  choiceCount: number;
  jumpCount: number;
  diagnosticCount: number;
  fileCount: number;
}

const STRESS_50K: StressConfig = {
  name: 'stress-50k-entities',
  entityCount: 50000,
  typeCount: 500,
  locationCount: 200,
  propertyCount: 10000,
  ruleCount: 1000,
  readCount: 5000,
  writeCount: 5000,
  exitCount: 500,
  choiceCount: 2000,
  jumpCount: 1500,
  diagnosticCount: 500,
  fileCount: 100,
};

const STRESS_DIAG: StressConfig = {
  name: 'stress-huge-diagnostics',
  entityCount: 1000,
  typeCount: 50,
  locationCount: 50,
  propertyCount: 500,
  ruleCount: 100,
  readCount: 500,
  writeCount: 500,
  exitCount: 100,
  choiceCount: 200,
  jumpCount: 100,
  diagnosticCount: 10000,
  fileCount: 50,
};

function generateFixture(config: StressConfig): object {
  const files = generateFiles(config.fileCount);
  const kinds = ['entity', 'location', 'type', 'property', 'rule', 'choice', 'exit', 'section'];
  const severities = ['error', 'warning', 'info'];
  const operators = ['==', '!=', '>=', '<=', '>', '<'];
  const valueKinds = ['integer', 'string', 'boolean', 'enum'];
  const propTypes = ['integer', 'string', 'boolean', 'enum', 'float'];

  // --- AST nodes ---
  const astNodes = [];
  astNodes.push({ kind: 'world', name: `Stress_${config.name}` });
  for (let i = 0; i < Math.min(config.entityCount, 5000); i++) {
    astNodes.push({ kind: 'entity', name: `Entity_${i}` });
  }
  for (let i = 0; i < config.locationCount; i++) {
    astNodes.push({ kind: 'location', name: `Location_${i}` });
  }

  // --- Symbol table entries ---
  const symbolEntries = [];
  for (let i = 0; i < config.entityCount; i++) {
    symbolEntries.push({
      id: `entity_${i}`,
      name: `Entity_${i}`,
      kind: 'entity',
      file: randomPick(files),
      line: randomInt(1, 500),
    });
  }
  for (let i = 0; i < config.locationCount; i++) {
    symbolEntries.push({
      id: `location_${i}`,
      name: `Location_${i}`,
      kind: 'location',
      file: randomPick(files),
      line: randomInt(1, 500),
    });
  }
  for (let i = 0; i < config.typeCount; i++) {
    symbolEntries.push({
      id: `type_${i}`,
      name: `Type_${i}`,
      kind: 'type',
      file: randomPick(files),
      line: randomInt(1, 500),
    });
  }

  // --- FactSet ---
  const reads = [];
  for (let i = 0; i < config.readCount; i++) {
    const file = randomPick(files);
    const line = randomInt(1, 500);
    reads.push({
      site: { kind: randomPick(['choice', 'rule'] as const), id: `site_r${i}` },
      entity_type: `Type_${randomInt(0, config.typeCount - 1)}`,
      property: `prop_${randomInt(0, config.propertyCount - 1)}`,
      operator: randomPick(operators),
      value_literal: String(randomInt(0, 100)),
      value_kind: randomPick(valueKinds),
      span: { file, start_line: line, start_col: 1, end_line: line, end_col: 40 },
    });
  }

  const writes = [];
  for (let i = 0; i < config.writeCount; i++) {
    const file = randomPick(files);
    const line = randomInt(1, 500);
    writes.push({
      site: { kind: randomPick(['choice', 'rule'] as const), id: `site_w${i}` },
      entity_type: `Type_${randomInt(0, config.typeCount - 1)}`,
      property: `prop_${randomInt(0, config.propertyCount - 1)}`,
      operator: '=',
      value_expr: String(randomInt(0, 100)),
      value_kind: randomPick(valueKinds),
      span: { file, start_line: line, start_col: 1, end_line: line, end_col: 40 },
    });
  }

  const exits = [];
  for (let i = 0; i < config.exitCount; i++) {
    const file = randomPick(files);
    const line = randomInt(1, 500);
    exits.push({
      from_location: `location_${randomInt(0, config.locationCount - 1)}`,
      to_location: `location_${randomInt(0, config.locationCount - 1)}`,
      exit_name: randomPick(['north', 'south', 'east', 'west', 'up', 'down']),
      is_conditional: random() > 0.7,
      guard_reads: [],
      span: { file, start_line: line, start_col: 1, end_line: line, end_col: 30 },
    });
  }

  const jumps = [];
  for (let i = 0; i < config.jumpCount; i++) {
    const file = randomPick(files);
    const line = randomInt(1, 500);
    jumps.push({
      from_section: `section_${randomInt(0, 999)}`,
      target: { kind: 'section', id: `section_${randomInt(0, 999)}` },
      span: { file, start_line: line, start_col: 1, end_line: line, end_col: 20 },
    });
  }

  const choices = [];
  for (let i = 0; i < config.choiceCount; i++) {
    const file = randomPick(files);
    const line = randomInt(1, 500);
    choices.push({
      section: `section_${randomInt(0, 999)}`,
      choice_id: `choice_${i}`,
      label: `${randomWord()} ${randomWord()}`,
      sticky: random() > 0.8,
      condition_reads: [],
      effect_writes: [],
      jump_indices: [],
      span: { file, start_line: line, start_col: 1, end_line: line, end_col: 50 },
    });
  }

  const rules = [];
  for (let i = 0; i < config.ruleCount; i++) {
    const file = randomPick(files);
    const line = randomInt(1, 500);
    rules.push({
      rule_id: `rule_${i}`,
      condition_reads: Array.from({ length: randomInt(1, 5) }, () => randomInt(0, config.readCount - 1)),
      effect_writes: Array.from({ length: randomInt(1, 3) }, () => randomInt(0, config.writeCount - 1)),
      span: { file, start_line: line, start_col: 1, end_line: line, end_col: 60 },
    });
  }

  // --- PropertyDependencyIndex ---
  const propEntries = [];
  for (let i = 0; i < config.propertyCount; i++) {
    const rc = randomInt(0, 10);
    const wc = randomInt(0, 5);
    propEntries.push({
      entity_type: `Type_${randomInt(0, config.typeCount - 1)}`,
      property: `prop_${i}`,
      read_count: rc,
      write_count: wc,
      read_indices: Array.from({ length: Math.min(rc, 3) }, () => randomInt(0, config.readCount - 1)),
      write_indices: Array.from({ length: Math.min(wc, 2) }, () => randomInt(0, config.writeCount - 1)),
      orphaned: wc === 0 && rc === 0 ? 'unused' : null,
    });
  }

  // --- DefinitionIndex ---
  const definitions = [];
  for (let i = 0; i < config.entityCount; i++) {
    const file = randomPick(files);
    const line = randomInt(1, 500);
    definitions.push({
      key: `entity_${i}`,
      span: { file, start_line: line, start_col: 1, end_line: line + randomInt(5, 50), end_col: 1 },
      definition: { kind: 'entity', type_name: `Type_${randomInt(0, config.typeCount - 1)}` },
    });
  }
  for (let i = 0; i < config.locationCount; i++) {
    const file = randomPick(files);
    const line = randomInt(1, 500);
    definitions.push({
      key: `location_${i}`,
      span: { file, start_line: line, start_col: 1, end_line: line + randomInt(3, 20), end_col: 1 },
      definition: { kind: 'location', display_name: `Location_${i}` },
    });
  }

  // --- UrdJson ---
  const entities = [];
  for (let i = 0; i < config.entityCount; i++) {
    const props: Record<string, unknown> = {};
    const propCount = randomInt(1, 5);
    for (let p = 0; p < propCount; p++) {
      props[`prop_${randomInt(0, config.propertyCount - 1)}`] = randomInt(0, 100);
    }
    entities.push({
      id: `entity_${i}`,
      name: `Entity_${i}`,
      properties: props,
      type: `Type_${randomInt(0, config.typeCount - 1)}`,
    });
  }

  const locations = [];
  for (let i = 0; i < config.locationCount; i++) {
    const locExits = [];
    const exitCount = randomInt(1, 4);
    for (let e = 0; e < exitCount; e++) {
      locExits.push({
        direction: randomPick(['north', 'south', 'east', 'west']),
        target: `location_${randomInt(0, config.locationCount - 1)}`,
      });
    }
    locations.push({
      id: `location_${i}`,
      name: `Location_${i}`,
      description: `A ${randomWord().toLowerCase()} ${randomWord().toLowerCase()} place`,
      exits: locExits,
    });
  }

  const types: Record<string, { properties: Record<string, { type: string }> }> = {};
  for (let i = 0; i < config.typeCount; i++) {
    const tProps: Record<string, { type: string }> = {};
    const pc = randomInt(2, 8);
    for (let p = 0; p < pc; p++) {
      tProps[`prop_${randomInt(0, config.propertyCount - 1)}`] = { type: randomPick(propTypes) };
    }
    types[`Type_${i}`] = { properties: tProps };
  }

  // --- Diagnostics ---
  const diagnostics = [];
  const diagCodes = ['E001', 'E002', 'E003', 'W001', 'W002', 'W003', 'I001', 'I002'];
  const diagMessages = [
    'Undefined reference',
    'Type mismatch',
    'Unreachable section',
    'Unused property',
    'Missing description',
    'Circular dependency detected',
    'Duplicate identifier',
    'Implicit type coercion',
  ];
  for (let i = 0; i < config.diagnosticCount; i++) {
    const file = randomPick(files);
    const line = randomInt(1, 500);
    diagnostics.push({
      severity: randomPick(severities),
      message: `${randomPick(diagMessages)}: ${randomWord()}_${randomInt(0, 999)}`,
      code: randomPick(diagCodes),
      span: { file, startLine: line, startCol: 1, endLine: line, endCol: 40 },
    });
  }

  // --- Assemble CompilerOutput ---
  const now = Date.now();

  return {
    header: {
      compileId: `stress-${config.name}`,
      timestamp: now,
      durationMs: randomInt(200, 2000),
      phaseTimings: [
        { phase: 'parse', durationMs: randomInt(50, 500) },
        { phase: 'resolve', durationMs: randomInt(50, 500) },
        { phase: 'analyse', durationMs: randomInt(50, 500) },
        { phase: 'lower', durationMs: randomInt(20, 200) },
        { phase: 'emit', durationMs: randomInt(10, 100) },
      ],
      worldCounts: {
        entities: config.entityCount,
        locations: config.locationCount,
        exits: config.exitCount,
        properties: config.propertyCount,
        rules: config.ruleCount,
      },
      inputFileCount: config.fileCount,
    },
    chunks: [
      {
        name: 'ast',
        data: { nodes: astNodes },
        contentHash: simpleHash(`ast-${config.name}`),
      },
      {
        name: 'symbolTable',
        data: { entries: symbolEntries },
        contentHash: simpleHash(`symbolTable-${config.name}`),
      },
      {
        name: 'factSet',
        data: { reads, writes, exits, jumps, choices, rules },
        contentHash: simpleHash(`factSet-${config.name}`),
      },
      {
        name: 'propertyDependencyIndex',
        data: {
          properties: propEntries,
          summary: {
            total_properties: config.propertyCount,
            total_reads: config.readCount,
            total_writes: config.writeCount,
            read_never_written: randomInt(10, 100),
            written_never_read: randomInt(5, 50),
          },
        },
        contentHash: simpleHash(`propDep-${config.name}`),
      },
      {
        name: 'definitionIndex',
        data: { definitions, count: definitions.length },
        contentHash: simpleHash(`defIndex-${config.name}`),
      },
      {
        name: 'urdJson',
        data: {
          world: { name: `Stress_${config.name}`, version: '1.0.0' },
          types,
          entities,
          locations,
        },
        contentHash: simpleHash(`urdJson-${config.name}`),
      },
      {
        name: 'diagnostics',
        data: diagnostics,
        contentHash: simpleHash(`diagnostics-${config.name}`),
      },
    ],
  };
}

// ===== Main =====

console.log('Generating stress fixtures...');

mkdirSync(fixturesDir, { recursive: true });
mkdirSync(publicFixturesDir, { recursive: true });

console.log('  Generating stress-50k-entities.json...');
const fixture50k = generateFixture(STRESS_50K);
const json50k = JSON.stringify(fixture50k);
const path50k = join(fixturesDir, 'stress-50k-entities.json');
writeFileSync(path50k, json50k);
writeFileSync(join(publicFixturesDir, 'stress-50k-entities.json'), json50k);
const size50k = (Buffer.byteLength(json50k) / 1024 / 1024).toFixed(1);
console.log(`  Written: ${path50k} (${size50k} MB)`);

console.log('  Generating stress-huge-diagnostics.json...');
const fixtureDiag = generateFixture(STRESS_DIAG);
const jsonDiag = JSON.stringify(fixtureDiag);
const pathDiag = join(fixturesDir, 'stress-huge-diagnostics.json');
writeFileSync(pathDiag, jsonDiag);
writeFileSync(join(publicFixturesDir, 'stress-huge-diagnostics.json'), jsonDiag);
const sizeDiag = (Buffer.byteLength(jsonDiag) / 1024 / 1024).toFixed(1);
console.log(`  Written: ${pathDiag} (${sizeDiag} MB)`);

console.log('Done!');
