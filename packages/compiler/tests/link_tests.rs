// Tests for Phase 3: LINK
//
// Test categories from the LINK phase brief:
// - Declaration collection
// - Choice-to-action compilation
// - Reference resolution
// - ID derivation
// - Integration
// - Error recovery with cascading suppression

// Placeholder test cases — to be expanded per the LINK phase brief:
//
// Collection sub-pass:
//   - Type registration
//   - Entity registration
//   - Location registration (slugified IDs)
//   - Section registration (file_stem/name IDs)
//   - Choice registration (section_id/slugified-label IDs)
//   - Action registration (choice-derived)
//   - Rule registration
//   - Sequence and phase registration
//   - Duplicate detection (all symbol categories)
//
// Resolution sub-pass:
//   - Entity references (`@name`)
//   - Type names on entities
//   - Property accesses (`@entity.property`)
//   - Jump targets (section-first priority rule)
//   - Exit destinations (slugified lookup)
//   - Containment keywords (`here`, `player`)
//   - Rule bound variables
//   - Visible scope enforcement (non-transitive imports)
//
// ID derivation:
//   - Entity IDs (used as declared)
//   - Section IDs (file_stem/section_name)
//   - Choice IDs (section_id/slugify(label))
//   - Location IDs (slugify(display_name))
//   - Action IDs (same as choice IDs for choice-derived)
//
// Error recovery:
//   - Unresolved reference → null annotation, no cascade
//   - Duplicate entity → first wins, diagnostic emitted
//   - Missing type → entity marked unresolved
