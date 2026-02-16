#!/usr/bin/env node
import { execFileSync } from 'node:child_process';
import { readdirSync } from 'node:fs';
import { resolve, basename, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const SCHEMA = resolve(ROOT, 'packages/schema/urd-world-schema.json');
const PASS_DIR = resolve(ROOT, 'tests/fixtures/json-schema/positive');
const FAIL_DIR = resolve(ROOT, 'tests/fixtures/json-schema/negative');

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

console.log(`=== Results: ${pass} passed, ${fail} failed ===`);
if (fail > 0) process.exit(1);
console.log('All tests passed.');
