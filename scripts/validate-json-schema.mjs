#!/usr/bin/env node
import { execFileSync } from 'node:child_process';
import { readdirSync, writeFileSync, unlinkSync } from 'node:fs';
import { resolve, dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { tmpdir } from 'node:os';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const SCHEMA = resolve(ROOT, 'packages/schema/urd-world-schema.json');
const PASS_DIR = resolve(ROOT, 'tests/fixtures/json-schema/positive');
const FAIL_DIR = resolve(ROOT, 'tests/fixtures/json-schema/negative');
const COMPILER_FIXTURES = resolve(ROOT, 'packages/compiler/tests/fixtures');

// Fixtures that should compile successfully (entry points only).
const COMPILER_POSITIVE = [
  'tavern-scene.urd.md',
  'monty-hall.urd.md',
  'two-room-key-puzzle.urd.md',
  'locked-garden.urd.md',
  'type-aliases.urd.md',
  'sunken-citadel.urd.md',
  'interrogation/main.urd.md',
];

function ajv(args) {
  return execFileSync('npx', ['ajv', ...args, '--spec=draft2020'], {
    stdio: 'pipe',
    encoding: 'utf-8',
    shell: true,
  });
}

let pass = 0;
let fail = 0;

// Compile schema
console.log('=== Compiling schema ===');
try {
  ajv(['compile', '-s', SCHEMA]);
  console.log('Schema compiles OK\n');
} catch {
  console.log('FAIL: Schema does not compile');
  process.exit(1);
}

// Positive tests
console.log('=== Positive tests (must validate) ===');
for (const file of readdirSync(PASS_DIR).filter(f => f.endsWith('.json')).sort()) {
  const path = resolve(PASS_DIR, file);
  try {
    ajv(['validate', '-s', SCHEMA, '-d', path]);
    console.log(`  PASS: ${file}`);
    pass++;
  } catch {
    console.log(`  FAIL: ${file} — should have passed`);
    fail++;
  }
}
console.log();

// Negative tests
console.log('=== Negative tests (must be rejected) ===');
for (const file of readdirSync(FAIL_DIR).filter(f => f.endsWith('.json')).sort()) {
  const path = resolve(FAIL_DIR, file);
  try {
    ajv(['validate', '-s', SCHEMA, '-d', path]);
    console.log(`  FAIL: ${file} — should have been rejected`);
    fail++;
  } catch {
    console.log(`  PASS: ${file} — correctly rejected`);
    pass++;
  }
}
console.log();

// Compiler output tests — compile each fixture, validate output against schema
console.log('=== Compiler output tests (compile → validate) ===');
const urdBin = resolve(ROOT, 'packages/compiler/target/release/urd');
const tmpFile = join(tmpdir(), `urd-schema-check-${process.pid}.json`);
let compilerPass = 0;
let compilerFail = 0;

for (const fixture of COMPILER_POSITIVE) {
  const fixturePath = resolve(COMPILER_FIXTURES, fixture);
  const label = fixture.replace('.urd.md', '');
  try {
    // Compile with the urd CLI (release binary)
    const json = execFileSync(urdBin, [fixturePath], {
      stdio: ['pipe', 'pipe', 'pipe'],
      encoding: 'utf-8',
    });

    // Write to temp file for ajv
    writeFileSync(tmpFile, json);

    // Validate against schema
    ajv(['validate', '-s', SCHEMA, '-d', tmpFile]);
    console.log(`  PASS: ${label}`);
    compilerPass++;
    pass++;
  } catch (err) {
    const msg = err.stderr ? err.stderr.trim().split('\n')[0] : err.message;
    console.log(`  FAIL: ${label} — ${msg}`);
    compilerFail++;
    fail++;
  }
}

// Clean up temp file
try { unlinkSync(tmpFile); } catch { /* ignore */ }
console.log();

console.log(`=== Results: ${pass} passed, ${fail} failed ===`);
if (fail > 0) process.exit(1);
console.log('All tests passed.');
