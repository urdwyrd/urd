// Tests for Phase 2: IMPORT
//
// Test categories from the IMPORT phase brief:
// - Path resolution
// - Graph construction
// - Topological sort determinism
// - Cycle detection
// - Error recovery

// Placeholder test cases — to be expanded per the IMPORT phase brief:
//
// Path resolution:
//   - Relative paths
//   - Forward-slash normalisation
//   - No `..` segments in resolved paths
//   - Case-sensitive comparison
//   - Casing mismatch detection (URD206 warning)
//
// Graph construction:
//   - Single file (no imports)
//   - Linear import chain
//   - Diamond dependency (A→B, A→C, B→D, C→D)
//   - Maximum depth (64 levels)
//
// Topological ordering:
//   - Dependencies processed before dependants
//   - Alphabetical tiebreaking at same depth
//   - Entry file always last
//
// Cycle detection:
//   - Direct cycle (A→B→A)
//   - Indirect cycle (A→B→C→A)
//   - Self-import
//
// Limits:
//   - File count limit (256 files, URD205)
//   - Import depth limit (64 levels, URD204)
//   - File size limit (1 MB, URD103)
//
// Error recovery:
//   - Missing import file (URD201)
//   - Failed import leaves no trace in graph
