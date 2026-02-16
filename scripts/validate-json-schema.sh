#!/bin/bash
set -euo pipefail

# Resolve repo root (works whether called from repo root or via pnpm from a package)
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

SCHEMA="$ROOT/packages/schema/urd-world-schema.json"
PASS_DIR="$ROOT/tests/fixtures/json-schema/positive"
FAIL_DIR="$ROOT/tests/fixtures/json-schema/negative"

PASS=0
FAIL=0

echo "=== Compiling schema ==="
if ! ajv compile -s "$SCHEMA" --spec=draft2020 2>/dev/null; then
  echo "FAIL: Schema does not compile"
  exit 1
fi
echo "Schema compiles OK"
echo ""

echo "=== Positive tests (must validate) ==="
for f in "$PASS_DIR"/*.json; do
  name=$(basename "$f")
  if ajv validate -s "$SCHEMA" -d "$f" --spec=draft2020 2>/dev/null; then
    echo "  PASS: $name"
    PASS=$((PASS + 1))
  else
    echo "  FAIL: $name — should have passed"
    FAIL=$((FAIL + 1))
  fi
done
echo ""

echo "=== Negative tests (must be rejected) ==="
for f in "$FAIL_DIR"/*.json; do
  name=$(basename "$f")
  if ajv validate -s "$SCHEMA" -d "$f" --spec=draft2020 2>/dev/null; then
    echo "  FAIL: $name — should have been rejected"
    FAIL=$((FAIL + 1))
  else
    echo "  PASS: $name — correctly rejected"
    PASS=$((PASS + 1))
  fi
done
echo ""

echo "=== Results: $PASS passed, $FAIL failed ==="

if [ "$FAIL" -gt 0 ]; then
  exit 1
fi

echo "All tests passed."
