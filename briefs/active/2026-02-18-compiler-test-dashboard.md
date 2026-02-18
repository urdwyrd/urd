# Urd Compiler — Test Dashboard JSON Report

*A brief for the test runner script and benchmark harness*

February 2026 | Engineering Phase

`cargo test → parse stdout → test-report.json → Astro dashboard`

## Execution Record

> **Instructions for AI:** Before this brief is moved to `briefs/done/`, fill in this section completely. Be specific and honest — this is the project's permanent record of what happened.

**Date completed:** *(fill in)*
**Status:** *(fill in)*

### What was done

*(fill in)*

### What changed from the brief

*(fill in)*

---

## Goal

Create a script that runs the Urd compiler's test suite and produces a structured 
JSON report. This JSON will later be consumed by the urd.dev Astro site to render 
a test dashboard.

## What to build

### 1. Test runner script: `scripts/compiler-test-report.js`

A Node.js script (no dependencies beyond what's in the repo) that:

1. Runs `cargo test --manifest-path packages/compiler/Cargo.toml` and captures output
2. Parses cargo test's stdout to extract:
   - Individual test names, pass/fail status, and timing
   - Total counts
   - Test grouping by phase (based on test module paths)
3. Outputs a JSON file to `packages/compiler/test-report.json`

### 2. Phase detection from test names

Cargo test outputs test names like:
```
test parse::parse_tests::frontmatter_basic ... ok
test import::import_tests::path_resolution_relative ... ok  
test link::link_tests::collection_type ... ok
test validate::validate_tests::property_default_type ... ok
test emit::emit_tests::world_block_basic ... ok
test slugify::slugify_tests::basic ... ok
```

Map these to phases:
- `parse::` → PARSE phase (diagnostic range 100-199)
- `import::` → IMPORT phase (diagnostic range 200-299)
- `link::` → LINK phase (diagnostic range 300-399)
- `validate::` → VALIDATE phase (diagnostic range 400-499)
- `emit::` → EMIT phase (diagnostic range 500-599)
- `slugify::` → group under LINK (it's a LINK utility)
- anything else → scaffolding

### 3. Category detection

Within each phase, group tests by their second-level module or by test name prefix.
For example, `link_tests::collection_type` and `link_tests::collection_entity` both 
belong to a "collection" category. Use the longest common prefix before the last 
underscore-separated word as the category name, or the explicit test group if cargo 
test provides it.

If heuristic grouping is unreliable, fall back to a flat list — accurate data is 
better than wrong grouping.

### 4. Diagnostic codes

The diagnostic codes per phase are static metadata (they don't change between test 
runs). Include them in the output as a fixed mapping:

- PARSE: URD100-URD112
- IMPORT: URD201-URD211
- LINK: URD301-URD314
- VALIDATE: URD401, URD402, URD404-URD420, URD422-URD428
- EMIT: (none — EMIT produces output, not diagnostics)

### 5. Architecture compliance checks

Include a static compliance array:
```json
[
  { "label": "Deterministic output", "status": "pass", "note": "Fixed key order, topological sort" },
  { "label": "Error recovery", "status": "pass", "note": "Mark & continue, no cascading" },
  { "label": "Phase contracts", "status": "pass", "note": "Input/output types enforced" },
  { "label": "Diagnostic ownership", "status": "pass", "note": "Each phase owns its code range" },
  { "label": "ID derivation", "status": "pass", "note": "Slugify, file_stem/name" },
  { "label": "Annotation model", "status": "pass", "note": "Option<String> IDs via symbol table" }
]
```

These are validated by the test suite. If tests fail, the overall status already 
reflects it.

### 6. JSON output schema

The output must conform to the schema at `scripts/compiler-test-report.schema.json`.
Copy the schema file from this task into that location.

The output file goes to: `packages/compiler/test-report.json`

### 7. pnpm script

Add to root `package.json`:
```json
{
  "scripts": {
    "compiler:test": "node scripts/compiler-test-report.js"
  }
}
```

Also add a convenience script that just runs tests without the report:
```json
{
  "scripts": {
    "compiler:test:raw": "cargo test --manifest-path packages/compiler/Cargo.toml"
  }
}
```

## What NOT to build

- No HTML output. JSON only.
- No dashboard UI. The Astro site will consume the JSON later.
- No npm dependencies. Use only Node.js built-ins (child_process, fs, path).
- No modification to any Rust code. This is a read-only consumer of cargo test output.

## Benchmarks: compilation performance

After the test suite runs, the script should also measure compilation performance.

### How it works

The compiler needs a small benchmark harness — a Rust binary that compiles a source 
file and reports per-phase timing to stdout as JSON. This is the one piece of Rust 
to create.

Create `packages/compiler/src/bin/bench.rs`:
- Accepts a `.urd.md` file path as a CLI argument
- Reads the source file
- Calls each phase in sequence, timing each one with `std::time::Instant`
- Prints a single JSON line to stdout:
  ```json
  {"name":"two_room_key_puzzle","source_bytes":1234,"output_bytes":5678,"total_ms":12.3,"parse_ms":2.1,"import_ms":0.5,"link_ms":3.2,"validate_ms":1.8,"emit_ms":4.7,"success":true,"diagnostic_count":0}
  ```
- Uses the existing `compile()` orchestrator but wraps each phase call with timing

### Benchmark files

Create a `packages/compiler/fixtures/` directory with the canonical test files as 
`.urd.md` sources. These are the same four files used in PARSE integration tests:
- `two_room_key_puzzle.urd.md`
- `tavern_scene.urd.md`
- `monty_hall.urd.md` (if available — skip if the source isn't in the test suite)
- `interrogation.urd.md` (if available — skip if not)

If the full source text for these files isn't readily available from the existing 
test code, create minimal representative files (at least two_room_key_puzzle, since 
the architecture brief has the complete worked example).

### Node.js integration

After running `cargo test`, the script:
1. Builds the bench binary: `cargo build --manifest-path packages/compiler/Cargo.toml --bin bench --release`
2. For each `.urd.md` file in `packages/compiler/fixtures/`:
   - Runs `./target/release/bench <file>` and captures stdout
   - Parses the JSON line
   - Adds it to the `benchmarks.files` array
3. Computes `benchmarks.aggregate` from the collected results

### Why release mode

Benchmarks must use `--release`. Debug builds are 10-50x slower for Rust and would 
produce misleading numbers. Test suite still runs in debug mode (faster compilation, 
debug assertions enabled).

## Output format example

```json
{
  "version": "1",
  "timestamp": "2026-02-18T14:30:00Z",
  "compiler": {
    "name": "urd-compiler",
    "version": "0.1.0",
    "language": "rust"
  },
  "summary": {
    "total_tests": 347,
    "passed": 347,
    "failed": 0,
    "skipped": 0,
    "duration_ms": 1234,
    "pass_rate": 1.0
  },
  "phases": [
    {
      "name": "parse",
      "diagnostic_range": "100-199",
      "tests": 76,
      "passed": 76,
      "failed": 0,
      "skipped": 0,
      "duration_ms": 320,
      "categories": [
        {
          "name": "frontmatter",
          "tests": 14,
          "passed": 14,
          "failed": 0,
          "test_names": [
            { "name": "frontmatter_basic", "status": "pass", "duration_ms": 2 },
            { "name": "frontmatter_types", "status": "pass", "duration_ms": 3 }
          ]
        }
      ],
      "diagnostics": ["URD100", "URD101", "URD102"]
    }
  ],
  "compliance": [
    { "label": "Deterministic output", "status": "pass", "note": "Fixed key order, topological sort" }
  ],
  "benchmarks": {
    "files": [
      {
        "name": "two_room_key_puzzle",
        "source_bytes": 1847,
        "output_bytes": 4231,
        "total_ms": 0.82,
        "phases": {
          "parse_ms": 0.21,
          "import_ms": 0.03,
          "link_ms": 0.18,
          "validate_ms": 0.14,
          "emit_ms": 0.26
        },
        "success": true,
        "diagnostic_count": 0
      }
    ],
    "aggregate": {
      "total_source_bytes": 1847,
      "total_output_bytes": 4231,
      "total_ms": 0.82,
      "files_compiled": 1,
      "files_succeeded": 1,
      "avg_ms_per_file": 0.82,
      "bytes_per_ms": 2252
    }
  }
}
```

## Testing the script

After implementation, run `pnpm compiler:test` and verify:
1. Exit code 0 when all tests pass
2. `packages/compiler/test-report.json` exists and is valid JSON
3. `summary.total_tests` matches the actual count from cargo test
4. Every test name from cargo output appears somewhere in the phases array
5. `summary.passed + summary.failed + summary.skipped == summary.total_tests`
