// Tests for Phase 4: VALIDATE
//
// Test categories from the VALIDATE phase brief:
// - Property type checking
// - Condition validation
// - Effect validation
// - Structural constraints
// - Skip rule verification
// - Integration tests

// Placeholder test cases — to be expanded per the VALIDATE phase brief:
//
// Property type checking:
//   - Boolean properties accept true/false
//   - Integer properties accept integers, reject strings
//   - Number properties accept integers and floats
//   - String properties accept strings
//   - Enum properties accept declared values, reject others
//   - Ref properties accept valid entity refs of correct type
//   - List properties validate element types
//
// Range and constraint enforcement:
//   - min/max on integer properties
//   - min/max on number properties
//   - min/max rejected on non-numeric types
//   - values only on enum type
//
// Condition validation:
//   - PropertyComparison operator/value type compatibility
//   - ContainmentCheck entity has correct traits
//   - ExhaustionCheck section exists
//
// Effect validation:
//   - Set: target property exists, value type compatible
//   - Move: entity has `portable` trait, destination has `container` trait
//   - Reveal: target property exists
//   - Destroy: target is an entity
//
// Trait validation:
//   - `portable` required for move targets
//   - `container` required for destinations
//   - `mobile` and `container` required for explicit @player
//
// Structural constraints:
//   - Action mutual exclusion (target vs target_type)
//   - Choice nesting depth (warn at 3, error at 4)
//   - world.start references existing location
//   - world.entry references existing sequence
//   - Sequence phase references valid actions/rules
//
// Skip rule:
//   - Null annotation → validation silently skipped
//   - No cascading errors from LINK failures
