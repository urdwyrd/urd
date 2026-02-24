/// Line-level cursor → identifier resolution.
///
/// Identifies what Urd construct is under the cursor using string-level
/// heuristics. Does NOT parse the Urd grammar — works on raw source lines.
/// Returns `None` for ambiguous or unrecognised positions.

/// A resolved reference under the cursor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reference {
    /// `@entity_id`
    Entity(String),
    /// `@entity_id.property`
    EntityProperty(String, String),
    /// `Type.property` in a condition or effect
    TypeProperty(String, String),
    /// `-> section_name` (a jump target)
    SectionJump(String),
    /// `== section_name` (a section label declaration)
    SectionLabel(String),
    /// `# Location Name` (a location heading)
    LocationHeading(String),
}

/// Identify the reference under the cursor at (line_text, col).
///
/// `col` is 0-indexed (LSP convention). Returns `None` if the cursor is
/// not on a recognisable identifier.
///
/// Precedence: `@entity.property` > `@entity` > `-> section` >
/// `== label` > `# heading` > `Type.property`.
pub fn identify_reference(line: &str, col: usize) -> Option<Reference> {
    let trimmed = line.trim_start();

    // 1. Check for section label: == name
    if trimmed.starts_with("== ") {
        let name = trimmed[3..].trim();
        if !name.is_empty() {
            return Some(Reference::SectionLabel(name.to_string()));
        }
    }
    if trimmed.starts_with("==") && trimmed.len() > 2 {
        let name = trimmed[2..].trim();
        if !name.is_empty() {
            return Some(Reference::SectionLabel(name.to_string()));
        }
    }

    // 2. Check for location heading: # Name (single #)
    if trimmed.starts_with("# ") && !trimmed.starts_with("## ") {
        let name = trimmed[2..].trim();
        if !name.is_empty() {
            return Some(Reference::LocationHeading(name.to_string()));
        }
    }

    // 3. Check for jump arrow: -> target
    if let Some(arrow_pos) = line.find("-> ") {
        let after_arrow = &line[arrow_pos + 3..];
        let target = after_arrow
            .split(|c: char| c.is_whitespace())
            .next()
            .unwrap_or("");
        if !target.is_empty() {
            let arrow_start = arrow_pos;
            let target_end = arrow_pos + 3 + target.len();
            if col >= arrow_start && col < target_end {
                // Strip @ prefix if jumping to an entity (-> @entity syntax)
                let clean = target.trim_start_matches('@');
                return Some(Reference::SectionJump(clean.to_string()));
            }
        }
    }

    // 4. Check for @entity or @entity.property
    if let Some(result) = find_entity_reference(line, col) {
        return Some(result);
    }

    // 5. Check for Type.property (uppercase start, dot, lowercase property)
    if let Some(result) = find_type_property(line, col) {
        return Some(result);
    }

    None
}

/// Find an @entity or @entity.property reference at the cursor position.
fn find_entity_reference(line: &str, col: usize) -> Option<Reference> {
    // Scan backwards from col to find '@', then forwards to get the full identifier
    let bytes = line.as_bytes();

    // Find the nearest '@' that could contain col
    for (i, &b) in bytes.iter().enumerate() {
        if b != b'@' {
            continue;
        }

        // Extract entity id: alphanumeric + underscore after '@'
        let id_start = i + 1;
        let id_end = line[id_start..]
            .find(|c: char| !c.is_alphanumeric() && c != '_')
            .map(|pos| id_start + pos)
            .unwrap_or(line.len());

        if id_start >= id_end {
            continue;
        }

        let entity_id = &line[id_start..id_end];

        // Check if there's a .property after the entity
        if id_end < line.len() && bytes[id_end] == b'.' {
            let prop_start = id_end + 1;
            let prop_end = line[prop_start..]
                .find(|c: char| !c.is_alphanumeric() && c != '_')
                .map(|pos| prop_start + pos)
                .unwrap_or(line.len());

            if prop_start < prop_end {
                let property = &line[prop_start..prop_end];
                // Cursor on property part?
                if col >= prop_start && col <= prop_end {
                    return Some(Reference::EntityProperty(
                        entity_id.to_string(),
                        property.to_string(),
                    ));
                }
                // Cursor on entity part (including @)?
                if col >= i && col < id_end {
                    return Some(Reference::EntityProperty(
                        entity_id.to_string(),
                        property.to_string(),
                    ));
                }
                // Cursor on the dot
                if col == id_end {
                    return Some(Reference::EntityProperty(
                        entity_id.to_string(),
                        property.to_string(),
                    ));
                }
            }
        }

        // Plain @entity reference
        if col >= i && col <= id_end {
            return Some(Reference::Entity(entity_id.to_string()));
        }
    }

    None
}

/// Find a Type.property reference at the cursor position.
///
/// Matches patterns like `Character.mood` where the first part starts
/// with an uppercase letter — distinguishing from entity refs.
fn find_type_property(line: &str, col: usize) -> Option<Reference> {
    // Look for word.word patterns
    let bytes = line.as_bytes();

    for (i, &b) in bytes.iter().enumerate() {
        if b != b'.' {
            continue;
        }

        // Extract word before dot
        let type_start = line[..i]
            .rfind(|c: char| !c.is_alphanumeric() && c != '_')
            .map(|pos| pos + 1)
            .unwrap_or(0);
        let type_name = &line[type_start..i];

        // Must start with uppercase (Type, not entity)
        if type_name.is_empty() || !type_name.chars().next().unwrap().is_uppercase() {
            continue;
        }

        // Extract word after dot
        let prop_start = i + 1;
        let prop_end = line[prop_start..]
            .find(|c: char| !c.is_alphanumeric() && c != '_')
            .map(|pos| prop_start + pos)
            .unwrap_or(line.len());

        if prop_start >= prop_end {
            continue;
        }

        let property = &line[prop_start..prop_end];

        // Check if cursor is within this token span
        if col >= type_start && col <= prop_end {
            return Some(Reference::TypeProperty(
                type_name.to_string(),
                property.to_string(),
            ));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entity_reference() {
        assert_eq!(
            identify_reference("[@warden]", 2),
            Some(Reference::Entity("warden".to_string()))
        );
    }

    #[test]
    fn entity_property_on_property() {
        assert_eq!(
            identify_reference("  ? @warden.trust >= 3", 14),
            Some(Reference::EntityProperty(
                "warden".to_string(),
                "trust".to_string()
            ))
        );
    }

    #[test]
    fn entity_property_on_entity_part() {
        assert_eq!(
            identify_reference("  ? @warden.trust >= 3", 6),
            Some(Reference::EntityProperty(
                "warden".to_string(),
                "trust".to_string()
            ))
        );
    }

    #[test]
    fn section_jump() {
        assert_eq!(
            identify_reference("  -> greet", 6),
            Some(Reference::SectionJump("greet".to_string()))
        );
    }

    #[test]
    fn section_label() {
        assert_eq!(
            identify_reference("== explore", 5),
            Some(Reference::SectionLabel("explore".to_string()))
        );
    }

    #[test]
    fn location_heading() {
        assert_eq!(
            identify_reference("# The Walled Garden", 5),
            Some(Reference::LocationHeading("The Walled Garden".to_string()))
        );
    }

    #[test]
    fn type_property() {
        assert_eq!(
            identify_reference("  Character.mood == friendly", 14),
            Some(Reference::TypeProperty(
                "Character".to_string(),
                "mood".to_string()
            ))
        );
    }

    #[test]
    fn plain_text_returns_none() {
        assert_eq!(identify_reference("A stone archway choked with ivy.", 10), None);
    }
}
