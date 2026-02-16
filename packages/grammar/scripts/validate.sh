#!/usr/bin/env bash
set -euo pipefail

# Urd Grammar Validation Script
# Runs the full test corpus against the parser.
# Can be invoked from repo root or from inside packages/grammar/.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GRAMMAR_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$GRAMMAR_DIR"

echo "Running grammar validation corpus..."
echo ""

# Run cargo test and capture output
if cargo test --test corpus 2>&1; then
    echo ""
    echo "════════════════════════════════════════"
    echo "  12/12 passed"
    echo "════════════════════════════════════════"
    exit 0
else
    echo ""
    echo "════════════════════════════════════════"
    echo "  FAILED — see output above"
    echo "════════════════════════════════════════"
    exit 1
fi
