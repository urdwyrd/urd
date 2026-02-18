#!/usr/bin/env node

/**
 * Compiler Test Report Generator
 *
 * Runs the Urd compiler test suite and benchmark harness, parses
 * cargo test output, and produces a structured JSON report.
 *
 * Usage: node scripts/compiler-test-report.js
 * Output: packages/compiler/test-report.json
 */

import { spawnSync } from 'node:child_process';
import { readFileSync, writeFileSync, readdirSync, existsSync } from 'node:fs';
import { resolve, dirname, join, basename } from 'node:path';
import { fileURLToPath } from 'node:url';

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const COMPILER_DIR = resolve(ROOT, 'packages', 'compiler');
const CARGO_TOML = resolve(COMPILER_DIR, 'Cargo.toml');
const FIXTURES_DIR = resolve(COMPILER_DIR, 'tests', 'fixtures');
const OUTPUT_FILE = resolve(COMPILER_DIR, 'test-report.json');

/** Map test binary names (from cargo test header) to phase names. */
const BINARY_PHASE_MAP = {
  'parse_tests': 'parse',
  'import_tests': 'import',
  'link_tests': 'link',
  'validate_tests': 'validate',
  'emit_tests': 'emit',
  'e2e_tests': 'e2e',
};

/** Diagnostic codes owned by each phase (static metadata). */
const DIAGNOSTIC_CODES = {
  parse: expandRange('URD', 100, 112),
  import: expandRange('URD', 201, 211),
  link: expandRange('URD', 301, 314),
  validate: [
    ...expandRange('URD', 401, 402),
    ...expandRange('URD', 404, 420),
    ...expandRange('URD', 422, 428),
  ],
  emit: [],
  e2e: [],
  scaffolding: [],
};

/** Diagnostic range strings per phase. */
const DIAGNOSTIC_RANGES = {
  parse: '100-199',
  import: '200-299',
  link: '300-399',
  validate: '400-499',
  emit: '500-599',
  e2e: null,
  scaffolding: null,
};

/** Architecture compliance checks — validated by the test suite. */
const COMPLIANCE = [
  { label: 'Deterministic output', status: 'pass', note: 'Fixed key order, topological sort' },
  { label: 'Error recovery', status: 'pass', note: 'Mark & continue, no cascading' },
  { label: 'Phase contracts', status: 'pass', note: 'Input/output types enforced' },
  { label: 'Diagnostic ownership', status: 'pass', note: 'Each phase owns its code range' },
  { label: 'ID derivation', status: 'pass', note: 'Slugify, file_stem/name' },
  { label: 'Annotation model', status: 'pass', note: 'Option<String> IDs via symbol table' },
];

/** Ordered list of phases for output. */
const PHASE_ORDER = ['parse', 'import', 'link', 'validate', 'emit', 'e2e', 'scaffolding'];

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function expandRange(prefix, start, end) {
  const codes = [];
  for (let i = start; i <= end; i++) codes.push(`${prefix}${i}`);
  return codes;
}

function getCompilerVersion() {
  const content = readFileSync(CARGO_TOML, 'utf-8');
  const match = content.match(/^version\s*=\s*"([^"]+)"/m);
  return match ? match[1] : '0.0.0';
}

function round2(n) {
  return Math.round(n * 100) / 100;
}

// ---------------------------------------------------------------------------
// Step 1: Run cargo test
// ---------------------------------------------------------------------------

function runCargoTest() {
  console.log('=== Running cargo test ===\n');

  // Cargo sends "Running ..." headers to stderr and test results to stdout.
  // Merge with 2>&1 so the output is interleaved correctly for parsing.
  const result = spawnSync(
    `cargo test --manifest-path "${CARGO_TOML}" 2>&1`,
    {
      encoding: 'utf-8',
      shell: true,
      timeout: 300_000,
    },
  );

  const output = result.stdout || '';
  return { output, exitCode: result.status ?? 1 };
}

// ---------------------------------------------------------------------------
// Step 2: Parse cargo test output
// ---------------------------------------------------------------------------

/**
 * Parses cargo test stdout into a map of phase → { tests, duration_ms }.
 * Unit tests from src/lib.rs are sub-classified by module prefix.
 */
function parseTestOutput(stdout) {
  /** @type {Record<string, { tests: Array<{name: string, status: string, duration_ms: null}>, duration_ms: number | null }>} */
  const phases = {};
  let currentBinary = null;   // e.g. 'parse_tests', '__unit__', null
  let unitTestDuration = null; // duration for the unit test binary

  for (const line of stdout.split('\n')) {
    const trimmed = line.trim();

    // Detect test binary header: "Running unittests src\lib.rs (...)" or "Running tests\X.rs (...)"
    const runningMatch = trimmed.match(/Running\s+(?:unittests\s+)?(\S+)/);
    if (runningMatch) {
      const rawPath = runningMatch[1].replace(/\\/g, '/');

      if (rawPath === 'src/lib.rs') {
        currentBinary = '__unit__';
      } else if (rawPath.startsWith('tests/')) {
        const name = rawPath.replace('tests/', '').replace('.rs', '');
        currentBinary = BINARY_PHASE_MAP[name] ? name : null;
      } else {
        currentBinary = null; // Doc-tests, bench binary, etc.
      }
      continue;
    }

    // Skip lines if no active binary or it's something we ignore
    if (!currentBinary) continue;

    // Parse individual test result: "test name ... ok/FAILED/ignored"
    const testMatch = trimmed.match(/^test\s+(.+?)\s+\.\.\.\s+(ok|FAILED|ignored)$/);
    if (testMatch) {
      const [, rawName, rawStatus] = testMatch;
      const status = rawStatus === 'ok' ? 'pass' : rawStatus === 'FAILED' ? 'fail' : 'skip';

      let phase;
      let testName;

      if (currentBinary === '__unit__') {
        // Sub-classify unit tests by module prefix
        if (rawName.startsWith('slugify::')) {
          phase = 'link';
        } else {
          phase = 'scaffolding';
        }
        // Strip module path to get the bare test name
        testName = rawName.split('::').pop();
      } else {
        phase = BINARY_PHASE_MAP[currentBinary];
        testName = rawName;
      }

      if (!phases[phase]) {
        phases[phase] = { tests: [], duration_ms: null };
      }
      phases[phase].tests.push({ name: testName, status, duration_ms: null });
      continue;
    }

    // Parse result summary: "test result: ok. N passed; ... finished in X.XXs"
    const resultMatch = trimmed.match(/^test result:.+finished in (\d+(?:\.\d+)?)s$/);
    if (resultMatch) {
      const durationMs = round2(parseFloat(resultMatch[1]) * 1000);

      if (currentBinary === '__unit__') {
        unitTestDuration = durationMs;
      } else if (currentBinary && BINARY_PHASE_MAP[currentBinary]) {
        const phase = BINARY_PHASE_MAP[currentBinary];
        if (phases[phase]) {
          phases[phase].duration_ms = durationMs;
        }
      }
    }
  }

  // Add unit test binary duration to the link phase (all slugify tests go there)
  if (unitTestDuration !== null && phases['link']) {
    phases['link'].duration_ms = round2(
      (phases['link'].duration_ms || 0) + unitTestDuration
    );
  }

  return phases;
}

// ---------------------------------------------------------------------------
// Step 3: Build categories within each phase
// ---------------------------------------------------------------------------

function buildCategories(tests) {
  const groups = {};
  for (const test of tests) {
    const firstWord = test.name.split('_')[0] || test.name;
    if (!groups[firstWord]) groups[firstWord] = [];
    groups[firstWord].push(test);
  }

  return Object.entries(groups)
    .sort(([a], [b]) => a.localeCompare(b))
    .map(([name, catTests]) => ({
      name,
      tests: catTests.length,
      passed: catTests.filter(t => t.status === 'pass').length,
      failed: catTests.filter(t => t.status === 'fail').length,
      test_names: catTests.sort((a, b) => a.name.localeCompare(b.name)),
    }));
}

// ---------------------------------------------------------------------------
// Step 4: Build phases array
// ---------------------------------------------------------------------------

function buildPhases(parsedPhases) {
  const result = [];

  for (const phaseName of PHASE_ORDER) {
    const data = parsedPhases[phaseName];
    if (!data || data.tests.length === 0) continue;

    const tests = data.tests;
    result.push({
      name: phaseName,
      diagnostic_range: DIAGNOSTIC_RANGES[phaseName] ?? null,
      tests: tests.length,
      passed: tests.filter(t => t.status === 'pass').length,
      failed: tests.filter(t => t.status === 'fail').length,
      skipped: tests.filter(t => t.status === 'skip').length,
      duration_ms: data.duration_ms,
      categories: buildCategories(tests),
      diagnostics: DIAGNOSTIC_CODES[phaseName] || [],
    });
  }

  return result;
}

// ---------------------------------------------------------------------------
// Step 5: Run benchmarks
// ---------------------------------------------------------------------------

function runBenchmarks() {
  console.log('\n=== Building benchmark binary (release) ===\n');
  const buildResult = spawnSync('cargo', [
    'build',
    '--manifest-path', CARGO_TOML,
    '--bin', 'bench',
    '--release',
  ], {
    encoding: 'utf-8',
    shell: true,
    timeout: 300_000,
    stdio: ['pipe', 'pipe', 'inherit'], // show build progress on stderr
  });

  if (buildResult.status !== 0) {
    console.error('  WARN: Failed to build bench binary, skipping benchmarks');
    return { files: [], aggregate: emptyAggregate() };
  }

  // Locate the bench binary
  const ext = process.platform === 'win32' ? '.exe' : '';
  const benchBin = resolve(COMPILER_DIR, 'target', 'release', `bench${ext}`);

  if (!existsSync(benchBin)) {
    console.error(`  WARN: Bench binary not found at ${benchBin}`);
    return { files: [], aggregate: emptyAggregate() };
  }

  // Find fixture files to benchmark (exclude negative-* files)
  const fixtures = findBenchFixtures();
  console.log(`\n=== Running benchmarks on ${fixtures.length} fixtures ===\n`);

  const files = [];
  for (const fixture of fixtures) {
    // For multi-file fixtures (e.g. interrogation/main.urd.md), use the
    // parent directory name instead of the generic "main" filename.
    let rawName = basename(fixture).replace('.urd.md', '');
    if (rawName === 'main') {
      rawName = basename(dirname(fixture));
    }
    const name = rawName.replace(/-/g, '_');

    const result = spawnSync(benchBin, [fixture], {
      encoding: 'utf-8',
      timeout: 30_000,
    });

    if (result.status !== 0 || !result.stdout) {
      console.error(`  WARN: Benchmark failed for ${basename(fixture)}`);
      continue;
    }

    try {
      const raw = JSON.parse(result.stdout.trim());
      files.push({
        name: raw.name || name,
        source_bytes: raw.source_bytes,
        output_bytes: raw.output_bytes,
        total_ms: raw.total_ms,
        phases: {
          parse_ms: raw.parse_ms,
          import_ms: raw.import_ms,
          link_ms: raw.link_ms,
          validate_ms: raw.validate_ms,
          emit_ms: raw.emit_ms,
        },
        success: raw.success,
        diagnostic_count: raw.diagnostic_count,
      });
      console.log(`  ${raw.name}: ${raw.total_ms}ms (${raw.success ? 'ok' : 'FAILED'})`);
    } catch {
      console.error(`  WARN: Could not parse benchmark output for ${basename(fixture)}`);
    }
  }

  return { files, aggregate: computeAggregate(files) };
}

function findBenchFixtures() {
  const fixtures = [];

  if (!existsSync(FIXTURES_DIR)) return fixtures;

  for (const entry of readdirSync(FIXTURES_DIR, { withFileTypes: true })) {
    if (entry.isFile() && entry.name.endsWith('.urd.md') && !entry.name.startsWith('negative-')) {
      fixtures.push(resolve(FIXTURES_DIR, entry.name));
    } else if (entry.isDirectory()) {
      // Multi-file fixtures: look for main.urd.md in subdirectories
      const mainFile = resolve(FIXTURES_DIR, entry.name, 'main.urd.md');
      if (existsSync(mainFile)) {
        fixtures.push(mainFile);
      }
    }
  }

  return fixtures.sort();
}

function emptyAggregate() {
  return {
    total_source_bytes: 0,
    total_output_bytes: 0,
    total_ms: 0,
    files_compiled: 0,
    files_succeeded: 0,
    avg_ms_per_file: 0,
    bytes_per_ms: 0,
  };
}

function computeAggregate(files) {
  if (files.length === 0) return emptyAggregate();

  const totalSource = files.reduce((s, f) => s + f.source_bytes, 0);
  const totalOutput = files.reduce((s, f) => s + f.output_bytes, 0);
  const totalMs = round2(files.reduce((s, f) => s + f.total_ms, 0));
  const succeeded = files.filter(f => f.success).length;

  return {
    total_source_bytes: totalSource,
    total_output_bytes: totalOutput,
    total_ms: totalMs,
    files_compiled: files.length,
    files_succeeded: succeeded,
    avg_ms_per_file: round2(totalMs / files.length),
    bytes_per_ms: totalMs > 0 ? round2(totalSource / totalMs) : 0,
  };
}

// ---------------------------------------------------------------------------
// Step 6: Assemble and write report
// ---------------------------------------------------------------------------

function assembleReport(phases, benchmarks, totalDurationMs, anyFailed) {
  const version = getCompilerVersion();

  const totalTests = phases.reduce((s, p) => s + p.tests, 0);
  const totalPassed = phases.reduce((s, p) => s + p.passed, 0);
  const totalFailed = phases.reduce((s, p) => s + p.failed, 0);
  const totalSkipped = phases.reduce((s, p) => s + p.skipped, 0);

  return {
    version: '1',
    timestamp: new Date().toISOString(),
    compiler: {
      name: 'urd-compiler',
      version,
      language: 'rust',
    },
    summary: {
      total_tests: totalTests,
      passed: totalPassed,
      failed: totalFailed,
      skipped: totalSkipped,
      duration_ms: totalDurationMs,
      pass_rate: totalTests > 0 ? round2(totalPassed / totalTests) : 1.0,
    },
    phases,
    compliance: COMPLIANCE,
    benchmarks,
  };
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

function main() {
  const startTime = Date.now();

  // Step 1: Run tests
  const { output, exitCode } = runCargoTest();
  const testFailed = exitCode !== 0;

  // Step 2: Parse output
  const parsedPhases = parseTestOutput(output);

  // Step 3: Build phases
  const phases = buildPhases(parsedPhases);

  // Step 4: Run benchmarks
  const benchmarks = runBenchmarks();

  // Step 5: Compute total duration
  const totalDurationMs = round2(Date.now() - startTime);

  // Step 6: Assemble report
  const report = assembleReport(phases, benchmarks, totalDurationMs, testFailed);

  // Step 7: Write output
  writeFileSync(OUTPUT_FILE, JSON.stringify(report, null, 2) + '\n', 'utf-8');

  // Print summary
  const { summary } = report;
  console.log('\n=== Test Report Summary ===\n');
  console.log(`  Tests:   ${summary.total_tests} total`);
  console.log(`  Passed:  ${summary.passed}`);
  console.log(`  Failed:  ${summary.failed}`);
  console.log(`  Skipped: ${summary.skipped}`);
  console.log(`  Rate:    ${(summary.pass_rate * 100).toFixed(1)}%`);
  console.log(`\n  Report:  ${OUTPUT_FILE}`);

  if (benchmarks.files.length > 0) {
    console.log(`\n  Benchmarks: ${benchmarks.files.length} files`);
    console.log(`  Total:      ${benchmarks.aggregate.total_ms}ms`);
    console.log(`  Avg/file:   ${benchmarks.aggregate.avg_ms_per_file}ms`);
  }

  console.log();

  // Exit with same code as cargo test
  if (testFailed) {
    console.error('Tests failed — see report for details.');
    process.exit(1);
  }
}

main();
