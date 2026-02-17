// Tests for Phase 5: EMIT
//
// Test categories from the EMIT phase brief:
// - World block assembly
// - Type block assembly
// - Entity block assembly
// - Location block assembly
// - Condition lowering
// - Effect lowering
// - Sequence advance modes
// - Dialogue assembly
// - Determinism
// - Integration tests

// Placeholder test cases — to be expanded per the EMIT phase brief:
//
// World block:
//   - `urd: "1"` always injected first
//   - `name` and `start` emitted from world block
//   - `entry` emitted when present
//   - Empty blocks omitted
//
// Type block:
//   - Types emitted in declaration order
//   - Properties with all fields (type, default, visibility, constraints)
//   - Traits array on type
//
// Entity block:
//   - Entities emitted in declaration order
//   - Type references
//   - Property overrides
//   - No player entity when not explicitly declared
//
// Location block:
//   - Location IDs from slugified heading names
//   - Exit objects with direction, destination, condition, blocked_message
//   - Contains lists from entity presence
//
// Condition lowering:
//   - PropertyComparison → string expression
//   - ContainmentCheck → `entity.container == location`
//   - `here` → `player.container`
//   - `player` → `player`
//   - Negated containment
//
// Effect lowering:
//   - Set → structured JSON object
//   - Move → structured JSON object
//   - Reveal → structured JSON object
//   - Destroy → structured JSON object
//
// Determinism:
//   - Same source produces byte-identical output
//   - Fixed key order (world, types, entities, locations, rules, actions, sequences, dialogue)
//   - Declaration-order iteration within blocks
//   - Topological-order across files
