/// Frontmatter sub-parser.
///
/// Handles the restricted YAML-like syntax between `---` delimiters.
/// Produces `Frontmatter` with `FrontmatterEntry` nodes.

use crate::ast::*;
use crate::graph::MAX_FRONTMATTER_NESTING_DEPTH;
use super::Parser;

/// Parse the frontmatter region between opening and closing `---` delimiters.
/// `start_line` is the first line after the opening `---`.
/// `end_line` is the line of the closing `---` (exclusive).
pub(crate) fn parse_frontmatter(
    parser: &mut Parser,
    start_line: usize,
    end_line: usize,
) -> Frontmatter {
    let span = if start_line < end_line {
        parser.span_lines(start_line, end_line.saturating_sub(1))
    } else {
        parser.line_span(start_line.min(parser.lines.len() - 1))
    };

    let mut entries = Vec::new();
    let mut i = start_line;

    while i < end_line {
        let raw_text = parser.lines[i].text;

        // Skip blank lines
        if raw_text.trim().is_empty() {
            i += 1;
            continue;
        }

        // Skip comment lines
        if raw_text.trim_start().starts_with('#') {
            i += 1;
            continue;
        }

        // Check for tabs
        let text = parser.check_tabs(i);
        let trimmed = text.trim();

        // Check nesting depth
        let indent_spaces = text.len() - text.trim_start().len();
        let nesting_level = indent_spaces / 2;
        if nesting_level > MAX_FRONTMATTER_NESTING_DEPTH {
            parser.diagnostics.error(
                "URD104",
                format!("Frontmatter nesting exceeds 8 levels at line {}.", parser.lines[i].line_number),
                parser.line_span(i),
            );
            i += 1;
            continue;
        }

        // Check for rejected YAML constructs
        if check_yaml_rejections(parser, i, trimmed) {
            i += 1;
            continue;
        }

        // Try to parse as a key-value entry
        match parse_entry(parser, &mut i, end_line, 0) {
            Some(entry) => entries.push(entry),
            None => {
                // URD111: Unrecognised frontmatter syntax
                let line_num = parser.lines[i].line_number;
                let display_text = truncate_for_display(raw_text.trim());
                parser.diagnostics.error(
                    "URD111",
                    format!("Unrecognised frontmatter syntax at line {}: '{}'.", line_num, display_text),
                    parser.line_span(i),
                );
                i += 1;
            }
        }
    }

    Frontmatter { entries, span }
}

/// Check for rejected YAML constructs and emit educational errors.
/// Returns true if the line was handled (rejected).
fn check_yaml_rejections(parser: &mut Parser, line_idx: usize, trimmed: &str) -> bool {
    // URD105: Anchors (&name)
    if trimmed.contains("&") && !trimmed.contains("&&") {
        // Check for YAML anchor pattern: &identifier (not in string values)
        if let Some(pos) = trimmed.find('&') {
            let after = &trimmed[pos + 1..];
            if !after.is_empty() && after.chars().next().map_or(false, |c| c.is_alphanumeric() || c == '_') {
                // Only flag if it looks like a standalone anchor, not inside a value
                let before = &trimmed[..pos];
                if before.is_empty() || before.ends_with(' ') || before.ends_with(':') {
                    parser.diagnostics.error(
                        "URD105",
                        "YAML anchors are not supported in Urd frontmatter. Define each value explicitly.",
                        parser.line_span(line_idx),
                    );
                    return true;
                }
            }
        }
    }

    // URD106: Aliases (*name) — but not choice lines
    if trimmed.starts_with('*') && !trimmed.starts_with("* ") && !trimmed.starts_with("*\t") {
        let after = &trimmed[1..];
        if !after.is_empty() && after.chars().next().map_or(false, |c| c.is_alphanumeric() || c == '_') {
            parser.diagnostics.error(
                "URD106",
                "YAML aliases are not supported in Urd frontmatter. Repeat the value where needed.",
                parser.line_span(line_idx),
            );
            return true;
        }
    }

    // URD107: Merge keys (<<:)
    if trimmed.starts_with("<<:") || trimmed == "<<" {
        parser.diagnostics.error(
            "URD107",
            "YAML merge keys are not supported in Urd frontmatter.",
            parser.line_span(line_idx),
        );
        return true;
    }

    // URD108: Custom tags (!!type)
    if trimmed.contains("!!") {
        parser.diagnostics.error(
            "URD108",
            "YAML custom tags are not supported in Urd frontmatter.",
            parser.line_span(line_idx),
        );
        return true;
    }

    // URD109: Block-style lists (- item) at the current indent level
    if trimmed.starts_with("- ") || trimmed == "-" {
        parser.diagnostics.error(
            "URD109",
            "Block-style lists are not supported. Use flow-style lists: [item1, item2].",
            parser.line_span(line_idx),
        );
        return true;
    }

    false
}

/// Parse a frontmatter entry (key: value).
/// Advances `i` past the consumed lines.
fn parse_entry(
    parser: &mut Parser,
    i: &mut usize,
    end_line: usize,
    _base_indent: usize,
) -> Option<FrontmatterEntry> {
    let line_idx = *i;
    let text = parser.check_tabs(line_idx);
    let indent_spaces = text.len() - text.trim_start().len();
    let nesting_level = indent_spaces / 2;
    let trimmed = text.trim();

    // Check nesting depth
    if nesting_level > MAX_FRONTMATTER_NESTING_DEPTH {
        parser.diagnostics.error(
            "URD104",
            format!("Frontmatter nesting exceeds 8 levels at line {}.", parser.lines[line_idx].line_number),
            parser.line_span(line_idx),
        );
        *i += 1;
        return None;
    }

    // Must have key: value pattern
    let colon_pos = trimmed.find(':')?;
    let key = trimmed[..colon_pos].trim();

    // Key must not be empty
    if key.is_empty() {
        return None;
    }

    let after_colon = trimmed[colon_pos + 1..].trim();
    let span = parser.line_span(line_idx);

    // Determine the value based on what follows the colon
    let value = if key == "import" {
        // Import declaration
        *i += 1;
        let path = after_colon.trim_matches('"').trim_matches('\'').to_string();
        FrontmatterValue::ImportDecl(ImportDecl {
            path,
            span: span.clone(),
        })
    } else if key == "world" {
        // World block — parse nested fields
        *i += 1;
        let fields = parse_world_fields(parser, i, end_line, indent_spaces + 2);
        let end_span = if *i > line_idx + 1 {
            parser.span_lines(line_idx, (*i).saturating_sub(1))
        } else {
            span.clone()
        };
        FrontmatterValue::WorldBlock(WorldBlock {
            fields,
            span: end_span,
        })
    } else if key == "types" {
        // Types block — parse nested type definitions
        *i += 1;
        let type_entries = parse_types_block(parser, i, end_line, indent_spaces + 2);
        FrontmatterValue::Map(type_entries)
    } else if key == "entities" {
        // Entities block — parse nested entity declarations
        *i += 1;
        let entity_entries = parse_entities_block(parser, i, end_line, indent_spaces + 2);
        FrontmatterValue::Map(entity_entries)
    } else if after_colon.is_empty() {
        // Block value — nested entries
        *i += 1;
        let child_indent = indent_spaces + 2;
        let children = parse_nested_entries(parser, i, end_line, child_indent);
        FrontmatterValue::Map(children)
    } else if after_colon.starts_with('{') {
        // Inline object
        *i += 1;
        let entries = parse_inline_object(after_colon);
        FrontmatterValue::InlineObject(entries.into_iter().map(|(k, v)| {
            FrontmatterEntry {
                key: k,
                value: FrontmatterValue::Scalar(v),
                span: span.clone(),
            }
        }).collect())
    } else if after_colon.starts_with('[') {
        // Flow-style list
        *i += 1;
        let items = parse_flow_list(after_colon);
        FrontmatterValue::List(items.into_iter().map(|s| {
            FrontmatterValue::Scalar(parse_scalar_value(&s))
        }).collect())
    } else if after_colon.starts_with("@") {
        // Entity reference or entity declaration
        *i += 1;
        // Check if it's an entity declaration: @name: Type { overrides }
        // For simple entity ref in a value position, store as scalar
        FrontmatterValue::Scalar(Scalar::String(after_colon.to_string()))
    } else {
        // Scalar value
        *i += 1;
        FrontmatterValue::Scalar(parse_scalar_value(after_colon))
    };

    Some(FrontmatterEntry {
        key: key.to_string(),
        value,
        span,
    })
}

/// Parse the world: block fields.
fn parse_world_fields(
    parser: &mut Parser,
    i: &mut usize,
    end_line: usize,
    child_indent: usize,
) -> Vec<(String, Scalar)> {
    let mut fields = Vec::new();

    while *i < end_line {
        let text = parser.check_tabs(*i);
        let trimmed = text.trim();

        if trimmed.is_empty() {
            *i += 1;
            continue;
        }

        let indent_spaces = text.len() - text.trim_start().len();
        if indent_spaces < child_indent {
            break; // Dedented — end of world block
        }

        if let Some(colon_pos) = trimmed.find(':') {
            let key = trimmed[..colon_pos].trim().to_string();
            let val = trimmed[colon_pos + 1..].trim();
            if !key.is_empty() {
                fields.push((key, parse_scalar_value(val)));
            }
        }

        *i += 1;
    }

    fields
}

/// Parse a types: block containing type definitions.
fn parse_types_block(
    parser: &mut Parser,
    i: &mut usize,
    end_line: usize,
    child_indent: usize,
) -> Vec<FrontmatterEntry> {
    let mut entries = Vec::new();

    while *i < end_line {
        let text = parser.check_tabs(*i);
        let trimmed = text.trim();

        if trimmed.is_empty() {
            *i += 1;
            continue;
        }

        if trimmed.starts_with('#') {
            *i += 1;
            continue;
        }

        let indent_spaces = text.len() - text.trim_start().len();
        if indent_spaces < child_indent {
            break;
        }

        // Try to parse as type definition: TypeName [traits]:
        if let Some(type_def) = parse_type_definition(parser, i, end_line, indent_spaces) {
            let span = type_def.span.clone();
            entries.push(FrontmatterEntry {
                key: type_def.name.clone(),
                value: FrontmatterValue::TypeDef(type_def),
                span,
            });
        } else {
            *i += 1;
        }
    }

    entries
}

/// Parse a type definition: `TypeName [trait1, trait2]:` followed by property lines.
fn parse_type_definition(
    parser: &mut Parser,
    i: &mut usize,
    end_line: usize,
    type_indent: usize,
) -> Option<TypeDef> {
    let line_idx = *i;
    let text = parser.check_tabs(line_idx);
    let trimmed = text.trim();

    // Must end with ':'
    if !trimmed.ends_with(':') {
        return None;
    }

    let without_colon = trimmed[..trimmed.len() - 1].trim();

    // Extract name and optional traits
    let (name, traits) = if let Some(bracket_start) = without_colon.find('[') {
        let bracket_end = without_colon.find(']')?;
        let name = without_colon[..bracket_start].trim().to_string();
        let traits_str = &without_colon[bracket_start + 1..bracket_end];
        let traits: Vec<String> = traits_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        (name, traits)
    } else {
        (without_colon.to_string(), Vec::new())
    };

    // Name must start with uppercase
    if name.is_empty() || !name.chars().next().map_or(false, |c| c.is_uppercase()) {
        return None;
    }

    let span_start = parser.line_span(line_idx);
    *i += 1;

    // Parse property definitions at deeper indent
    let prop_indent = type_indent + 2;
    let mut properties = Vec::new();

    while *i < end_line {
        let ptext = parser.check_tabs(*i);
        let ptrimmed = ptext.trim();

        if ptrimmed.is_empty() {
            *i += 1;
            continue;
        }

        let pindent = ptext.len() - ptext.trim_start().len();
        if pindent < prop_indent {
            break;
        }

        if let Some(prop) = parse_property_def(parser, *i) {
            properties.push(prop);
        }
        *i += 1;
    }

    let span = if *i > line_idx + 1 {
        parser.span_lines(line_idx, (*i).saturating_sub(1))
    } else {
        span_start
    };

    Some(TypeDef {
        name,
        traits,
        properties,
        span,
    })
}

/// Parse a property definition line: `name: type` or `~name: type = default`.
fn parse_property_def(parser: &mut Parser, line_idx: usize) -> Option<PropertyDef> {
    let text = parser.check_tabs(line_idx);
    let trimmed = text.trim();
    let span = parser.line_span(line_idx);

    // Check for hidden prefix
    let (is_hidden, rest) = if trimmed.starts_with('~') {
        (true, &trimmed[1..])
    } else {
        (false, trimmed)
    };

    // Split on first colon
    let colon_pos = rest.find(':')?;
    let name = rest[..colon_pos].trim().to_string();
    let type_and_default = rest[colon_pos + 1..].trim();

    // Split on ' = ' for default value
    let (type_str, default) = if let Some(eq_pos) = type_and_default.find(" = ") {
        let t = type_and_default[..eq_pos].trim();
        let d = type_and_default[eq_pos + 3..].trim();
        (t, Some(parse_scalar_value(d)))
    } else {
        (type_and_default, None)
    };

    // Parse the type
    let (property_type, values, ref_type, element_type, element_values, element_ref_type, min, max) =
        parse_type_signature(type_str);

    Some(PropertyDef {
        name,
        property_type,
        default,
        visibility: if is_hidden { Some("hidden".to_string()) } else { None },
        values,
        min,
        max,
        ref_type,
        element_type,
        element_values,
        element_ref_type,
        description: None,
        span,
    })
}

/// Parse a type signature like `string`, `enum(a, b)`, `ref(Type)`, `list(ref(Type))`.
#[allow(clippy::type_complexity)]
fn parse_type_signature(
    type_str: &str,
) -> (
    String,                    // property_type
    Option<Vec<String>>,       // values (for enum)
    Option<String>,            // ref_type
    Option<String>,            // element_type
    Option<Vec<String>>,       // element_values
    Option<String>,            // element_ref_type
    Option<f64>,               // min
    Option<f64>,               // max
) {
    let type_str = type_str.trim();

    if type_str.starts_with("enum(") && type_str.ends_with(')') {
        let inner = &type_str[5..type_str.len() - 1];
        let values: Vec<String> = inner.split(',').map(|s| s.trim().to_string()).collect();
        return ("enum".to_string(), Some(values), None, None, None, None, None, None);
    }

    if type_str.starts_with("ref(") && type_str.ends_with(')') {
        let inner = &type_str[4..type_str.len() - 1].trim();
        return ("ref".to_string(), None, Some(inner.to_string()), None, None, None, None, None);
    }

    if type_str.starts_with("list(") && type_str.ends_with(')') {
        let inner = &type_str[5..type_str.len() - 1].trim();

        // Parse element type recursively
        if inner.starts_with("enum(") && inner.ends_with(')') {
            let enum_inner = &inner[5..inner.len() - 1];
            let values: Vec<String> = enum_inner.split(',').map(|s| s.trim().to_string()).collect();
            return ("list".to_string(), None, None, Some("enum".to_string()), Some(values), None, None, None);
        }
        if inner.starts_with("ref(") && inner.ends_with(')') {
            let ref_inner = &inner[4..inner.len() - 1].trim();
            return ("list".to_string(), None, None, Some("ref".to_string()), None, Some(ref_inner.to_string()), None, None);
        }

        return ("list".to_string(), None, None, Some(inner.to_string()), None, None, None, None);
    }

    // Simple types: string, bool, integer, number
    (type_str.to_string(), None, None, None, None, None, None, None)
}

/// Parse an entities: block.
fn parse_entities_block(
    parser: &mut Parser,
    i: &mut usize,
    end_line: usize,
    child_indent: usize,
) -> Vec<FrontmatterEntry> {
    let mut entries = Vec::new();

    while *i < end_line {
        let text = parser.check_tabs(*i);
        let trimmed = text.trim();

        if trimmed.is_empty() {
            *i += 1;
            continue;
        }

        if trimmed.starts_with('#') {
            *i += 1;
            continue;
        }

        let indent_spaces = text.len() - text.trim_start().len();
        if indent_spaces < child_indent {
            break;
        }

        if let Some(entity) = parse_entity_declaration(parser, *i) {
            let span = entity.span.clone();
            entries.push(FrontmatterEntry {
                key: entity.id.clone(),
                value: FrontmatterValue::EntityDecl(entity),
                span,
            });
        }
        *i += 1;
    }

    entries
}

/// Parse an entity declaration: `@name: Type { overrides }`.
fn parse_entity_declaration(parser: &mut Parser, line_idx: usize) -> Option<EntityDecl> {
    let text = parser.check_tabs(line_idx);
    let trimmed = text.trim();
    let span = parser.line_span(line_idx);

    // Must start with @
    if !trimmed.starts_with('@') {
        return None;
    }

    let rest = &trimmed[1..];

    // Find the colon separator
    let colon_pos = rest.find(':')?;
    let id = rest[..colon_pos].trim().to_string();
    let after_colon = rest[colon_pos + 1..].trim();

    // Extract type name and optional overrides
    let (type_name, overrides) = if let Some(brace_start) = after_colon.find('{') {
        let type_name = after_colon[..brace_start].trim().to_string();
        let brace_end = after_colon.rfind('}')?;
        let overrides_str = &after_colon[brace_start + 1..brace_end];
        let overrides = parse_inline_object(&format!("{{{}}}", overrides_str));
        (type_name, overrides)
    } else {
        (after_colon.to_string(), Vec::new())
    };

    Some(EntityDecl {
        id,
        type_name,
        property_overrides: overrides,
        annotation: None,
        span,
    })
}

/// Parse nested frontmatter entries at a given indent level.
fn parse_nested_entries(
    parser: &mut Parser,
    i: &mut usize,
    end_line: usize,
    child_indent: usize,
) -> Vec<FrontmatterEntry> {
    let mut entries = Vec::new();

    while *i < end_line {
        let text = parser.check_tabs(*i);
        let trimmed = text.trim();

        if trimmed.is_empty() {
            *i += 1;
            continue;
        }

        let indent_spaces = text.len() - text.trim_start().len();
        if indent_spaces < child_indent {
            break;
        }

        match parse_entry(parser, i, end_line, child_indent) {
            Some(entry) => entries.push(entry),
            None => {
                *i += 1;
            }
        }
    }

    entries
}

/// Parse an inline object: `{ key: value, key2: value2 }`.
fn parse_inline_object(s: &str) -> Vec<(String, Scalar)> {
    let inner = s.trim();
    let inner = if inner.starts_with('{') && inner.ends_with('}') {
        &inner[1..inner.len() - 1]
    } else {
        inner
    };

    let mut result = Vec::new();
    for pair in split_top_level(inner, ',') {
        let pair = pair.trim();
        if let Some(colon_pos) = pair.find(':') {
            let key = pair[..colon_pos].trim().to_string();
            let val = pair[colon_pos + 1..].trim();
            if !key.is_empty() {
                result.push((key, parse_scalar_value(val)));
            }
        }
    }
    result
}

/// Parse a flow-style list: `[a, b, c]`.
fn parse_flow_list(s: &str) -> Vec<String> {
    let inner = s.trim();
    let inner = if inner.starts_with('[') && inner.ends_with(']') {
        &inner[1..inner.len() - 1]
    } else {
        inner
    };

    split_top_level(inner, ',')
        .into_iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Split a string by a delimiter, respecting nested brackets and quotes.
fn split_top_level(s: &str, delimiter: char) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut depth = 0;
    let mut in_quotes = false;
    let mut quote_char = '"';

    for ch in s.chars() {
        if in_quotes {
            current.push(ch);
            if ch == quote_char {
                in_quotes = false;
            }
            continue;
        }

        match ch {
            '"' | '\'' => {
                in_quotes = true;
                quote_char = ch;
                current.push(ch);
            }
            '(' | '[' | '{' => {
                depth += 1;
                current.push(ch);
            }
            ')' | ']' | '}' => {
                depth -= 1;
                current.push(ch);
            }
            c if c == delimiter && depth == 0 => {
                result.push(current.clone());
                current.clear();
            }
            _ => {
                current.push(ch);
            }
        }
    }

    if !current.is_empty() {
        result.push(current);
    }

    result
}

/// Parse a scalar value from a string.
pub(crate) fn parse_scalar_value(s: &str) -> Scalar {
    let s = s.trim();

    // Boolean
    if s == "true" {
        return Scalar::Boolean(true);
    }
    if s == "false" {
        return Scalar::Boolean(false);
    }

    // Quoted string
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        return Scalar::String(s[1..s.len() - 1].to_string());
    }

    // Integer
    if let Ok(i) = s.parse::<i64>() {
        return Scalar::Integer(i);
    }

    // Number (float)
    if let Ok(f) = s.parse::<f64>() {
        return Scalar::Number(f);
    }

    // Default: unquoted string
    Scalar::String(s.to_string())
}

/// Truncate text for display in diagnostic messages.
fn truncate_for_display(text: &str) -> String {
    if text.len() > 60 {
        format!("{}...", &text[..57])
    } else {
        text.to_string()
    }
}
