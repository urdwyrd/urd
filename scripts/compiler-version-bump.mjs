#!/usr/bin/env node

/**
 * Compiler Version Bump
 *
 * Bumps the Urd compiler version in Cargo.toml, regenerates the test
 * report, and copies it to the site data directory. The version flows
 * from Cargo.toml to everything else:
 *
 *   Cargo.toml → env!("CARGO_PKG_VERSION") → WASM compiler_version()
 *   Cargo.toml → test-report.json → CompilerStatus island
 *   Cargo.toml → playground footer (via WASM at runtime)
 *
 * Usage:
 *   node scripts/compiler-version-bump.mjs           # patch bump (0.1.0 → 0.1.1)
 *   node scripts/compiler-version-bump.mjs minor     # minor bump (0.1.0 → 0.2.0)
 *   node scripts/compiler-version-bump.mjs major     # major bump (0.1.0 → 1.0.0)
 *   node scripts/compiler-version-bump.mjs 0.3.0     # explicit version
 *
 * What it does:
 *   1. Reads the current version from Cargo.toml
 *   2. Computes the new version (semver bump or explicit)
 *   3. Writes the new version to Cargo.toml
 *   4. Runs the test suite (cargo test)
 *   5. Regenerates the test report (with the new version)
 *   6. Copies the report to the site data directory
 *
 * The script aborts if tests fail — the version is still written to
 * Cargo.toml so you can fix the issue and re-run, but the report is
 * not updated until tests pass.
 */

import { readFileSync, writeFileSync, cpSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const CARGO_TOML = resolve(ROOT, 'packages', 'compiler', 'Cargo.toml');
const TEST_REPORT = resolve(ROOT, 'packages', 'compiler', 'test-report.json');
const SITE_REPORT = resolve(ROOT, 'sites', 'urd.dev', 'src', 'data', 'compiler-test-report.json');

// ---------------------------------------------------------------------------
// 1. Read current version
// ---------------------------------------------------------------------------

const cargoContent = readFileSync(CARGO_TOML, 'utf-8');
const versionMatch = cargoContent.match(/^version\s*=\s*"([^"]+)"/m);

if (!versionMatch) {
  console.error('Could not find version in Cargo.toml');
  process.exit(1);
}

const current = versionMatch[1];
const [major, minor, patch] = current.split('.').map(Number);

// ---------------------------------------------------------------------------
// 2. Compute new version
// ---------------------------------------------------------------------------

const arg = process.argv[2] || 'patch';
let next;

if (arg === 'patch') {
  next = `${major}.${minor}.${patch + 1}`;
} else if (arg === 'minor') {
  next = `${major}.${minor + 1}.0`;
} else if (arg === 'major') {
  next = `${major + 1}.0.0`;
} else if (/^\d+\.\d+\.\d+$/.test(arg)) {
  next = arg;
} else {
  console.error(`Invalid argument: "${arg}". Use patch, minor, major, or an explicit version (e.g. 0.2.0).`);
  process.exit(1);
}

if (next === current) {
  console.log(`Version is already ${current}. Nothing to do.`);
  process.exit(0);
}

console.log(`\n  ${current} → ${next}\n`);

// ---------------------------------------------------------------------------
// 3. Write new version to Cargo.toml
// ---------------------------------------------------------------------------

const updated = cargoContent.replace(
  /^(version\s*=\s*)"[^"]+"/m,
  `$1"${next}"`
);

writeFileSync(CARGO_TOML, updated, 'utf-8');
console.log(`  ✓ Updated Cargo.toml`);

// ---------------------------------------------------------------------------
// 4. Run tests
// ---------------------------------------------------------------------------

console.log(`\n  Running cargo test...\n`);

const testResult = spawnSync('cargo', [
  'test',
  '--manifest-path', CARGO_TOML,
], {
  encoding: 'utf-8',
  shell: true,
  timeout: 300_000,
  stdio: 'inherit',
});

if (testResult.status !== 0) {
  console.error(`\n  Tests failed. Cargo.toml has been updated to ${next} but the`);
  console.error(`  test report has NOT been regenerated. Fix the failing tests and`);
  console.error(`  re-run this script, or run 'pnpm compiler:test' manually.\n`);
  process.exit(1);
}

// ---------------------------------------------------------------------------
// 5. Regenerate test report
// ---------------------------------------------------------------------------

console.log(`\n  Regenerating test report...\n`);

const reportResult = spawnSync('node', [
  resolve(ROOT, 'scripts', 'compiler-test-report.mjs'),
], {
  encoding: 'utf-8',
  shell: true,
  timeout: 600_000,
  stdio: 'inherit',
});

if (reportResult.status !== 0) {
  console.error(`\n  Test report generation failed.\n`);
  process.exit(1);
}

// ---------------------------------------------------------------------------
// 6. Copy report to site data
// ---------------------------------------------------------------------------

cpSync(TEST_REPORT, SITE_REPORT);
console.log(`  ✓ Copied report to site data`);

// ---------------------------------------------------------------------------
// Done
// ---------------------------------------------------------------------------

console.log(`\n  urd-compiler ${current} → ${next}`);
console.log(`\n  Files updated:`);
console.log(`    packages/compiler/Cargo.toml`);
console.log(`    packages/compiler/test-report.json`);
console.log(`    sites/urd.dev/src/data/compiler-test-report.json\n`);
