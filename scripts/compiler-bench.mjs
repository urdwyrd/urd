/**
 * Benchmark harness — runs the release-built bench binary against every
 * .urd.md fixture, prints a summary table, and updates the benchmarks
 * section in test-report.json.
 */

import { execSync } from 'node:child_process';
import { readFileSync, writeFileSync, readdirSync, statSync, existsSync } from 'node:fs';
import { join, resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const FIXTURES = resolve(ROOT, 'packages/compiler/tests/fixtures');
const BENCH = resolve(ROOT, 'packages/compiler/target/release/bench');
const REPORT_FILE = resolve(ROOT, 'packages/compiler/test-report.json');
const SITE_COPY = resolve(ROOT, 'sites/urd.dev/src/data/compiler-test-report.json');

/** Recursively collect all .urd.md files under a directory (excludes negative-* fixtures). */
function collectFixtures(dir) {
  const files = [];
  for (const entry of readdirSync(dir)) {
    const full = join(dir, entry);
    if (statSync(full).isDirectory()) {
      files.push(...collectFixtures(full));
    } else if (entry.endsWith('.urd.md') && !entry.startsWith('negative-')) {
      files.push(full);
    }
  }
  return files;
}

function round2(n) {
  return Math.round(n * 100) / 100;
}

const fixtures = collectFixtures(FIXTURES).sort();

if (fixtures.length === 0) {
  console.error('No .urd.md fixtures found in', FIXTURES);
  process.exit(1);
}

console.log(`Benchmarking ${fixtures.length} fixtures (release build)\n`);

const results = [];

for (const file of fixtures) {
  try {
    const raw = execSync(`"${BENCH}" "${file}"`, { encoding: 'utf-8' }).trim();
    const data = JSON.parse(raw);
    results.push({
      name: data.name,
      source_bytes: data.source_bytes,
      output_bytes: data.output_bytes,
      total_ms: data.total_ms,
      phases: {
        parse_ms: data.parse_ms,
        import_ms: data.import_ms,
        link_ms: data.link_ms,
        validate_ms: data.validate_ms,
        emit_ms: data.emit_ms,
      },
      success: data.success,
      diagnostic_count: data.diagnostic_count,
    });
  } catch (e) {
    console.error(`FAIL: ${file}`);
    console.error(e.stderr || e.message);
  }
}

// ── Print summary table ────────────────────────────────────────────────────

const pad = (s, n) => String(s).padStart(n);
const nameW = Math.max(20, ...results.map(r => r.name.length)) + 2;

console.log(
  'Name'.padEnd(nameW) +
  pad('Parse', 10) + pad('Import', 10) + pad('Link', 10) +
  pad('Validate', 10) + pad('Emit', 10) + pad('Total', 10) +
  '  Result'
);
console.log('─'.repeat(nameW + 62));

for (const r of results) {
  const ms = (v) => pad(v.toFixed(3), 8) + 'ms';
  console.log(
    r.name.padEnd(nameW) +
    ms(r.phases.parse_ms) + ms(r.phases.import_ms) + ms(r.phases.link_ms) +
    ms(r.phases.validate_ms) + ms(r.phases.emit_ms) + ms(r.total_ms) +
    (r.success ? '  OK' : '  FAIL')
  );
}

if (results.length > 1) {
  const sum = (key) => round2(results.reduce((a, r) => a + r.phases[key], 0));
  const totalMs = round2(results.reduce((a, r) => a + r.total_ms, 0));
  const ms = (v) => pad(v.toFixed(3), 8) + 'ms';
  console.log('─'.repeat(nameW + 62));
  console.log(
    'TOTAL'.padEnd(nameW) +
    ms(sum('parse_ms')) + ms(sum('import_ms')) + ms(sum('link_ms')) +
    ms(sum('validate_ms')) + ms(sum('emit_ms')) + ms(totalMs) +
    `  ${results.filter(r => r.success).length}/${results.length} OK`
  );
}

// ── Update test-report.json ────────────────────────────────────────────────

if (existsSync(REPORT_FILE)) {
  const totalSource = results.reduce((s, f) => s + f.source_bytes, 0);
  const totalOutput = results.reduce((s, f) => s + f.output_bytes, 0);
  const totalMs = round2(results.reduce((s, f) => s + f.total_ms, 0));
  const succeeded = results.filter(f => f.success).length;

  const benchmarks = {
    files: results,
    aggregate: {
      total_source_bytes: totalSource,
      total_output_bytes: totalOutput,
      total_ms: totalMs,
      files_compiled: results.length,
      files_succeeded: succeeded,
      avg_ms_per_file: round2(totalMs / results.length),
      bytes_per_ms: totalMs > 0 ? round2(totalSource / totalMs) : 0,
    },
  };

  const report = JSON.parse(readFileSync(REPORT_FILE, 'utf-8'));
  report.benchmarks = benchmarks;
  const json = JSON.stringify(report, null, 2) + '\n';
  writeFileSync(REPORT_FILE, json, 'utf-8');
  if (existsSync(SITE_COPY)) {
    writeFileSync(SITE_COPY, json, 'utf-8');
  }
  console.log(`\nUpdated benchmarks in test-report.json`);
} else {
  console.log(`\nNo report found at ${REPORT_FILE} — skipping update`);
}
