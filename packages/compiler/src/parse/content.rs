/// Narrative content parser.
///
/// Implements the PEG grammar's Block dispatch for the content region
/// of a `.urd.md` file. Processes lines sequentially, matching each
/// against the ordered rule list.

use crate::ast::*;
use crate::span::Span;
use super::Parser;

/// Parse narrative content lines starting from `parser.current_line`.
/// `min_indent` is the minimum indent level for content to be parsed
/// as children (used for choice nesting).
pub(crate) fn parse_content(parser: &mut Parser, min_indent: usize) -> Vec<ContentNode> {
    let mut nodes = Vec::new();

    while !parser.at_end() {
        let line = match parser.peek_line() {
            Some(l) => l,
            None => break,
        };

        // Skip blank lines
        if line.trim().is_empty() {
            parser.current_line += 1;
            continue;
        }

        // Process tabs and measure indent
        let processed = parser.check_tabs(parser.current_line);
        let (indent_level, _) = Parser::measure_indent(&processed);

        // If we're in a nested context (min_indent > 0) and this line
        // is at or below the minimum indent, stop — we've left the parent block
        if min_indent > 0 && indent_level < min_indent {
            break;
        }

        // Dispatch to block parsers
        match parse_block(parser, &processed, indent_level) {
            Some(node) => nodes.push(node),
            None => {
                // Should not happen — Prose is the fallback
                parser.current_line += 1;
            }
        }
    }

    nodes
}

/// Block dispatch: try each alternative in order.
/// Returns the parsed content node. Advances parser.current_line.
fn parse_block(
    parser: &mut Parser,
    processed_line: &str,
    indent_level: usize,
) -> Option<ContentNode> {
    let (_, rest) = Parser::measure_indent(processed_line);
    let line_idx = parser.current_line;

    // 1. OrConditionBlock: `? any:`
    if rest.starts_with("? any:") {
        return Some(parse_or_condition_block(parser, indent_level));
    }

    // 2. RuleBlock: `rule name:`
    if rest.starts_with("rule ") && rest.ends_with(':') {
        return Some(parse_rule_block(parser, indent_level));
    }

    // 3. Headings: ### before ## before #
    if rest.starts_with("### ") {
        return Some(parse_phase_heading(parser));
    }
    if rest.starts_with("## ") {
        return Some(parse_sequence_heading(parser));
    }
    if rest.starts_with("# ") {
        return Some(parse_location_heading(parser));
    }

    // 4. SectionLabel: == name
    if rest.starts_with("== ") {
        return Some(parse_section_label(parser));
    }

    // 5. EntityLine: @entity: speech or @entity action
    if rest.starts_with('@') && !rest.starts_with("[@") {
        return parse_entity_line(parser, indent_level);
    }

    // 6. ArrowLine: -> exit declaration or jump
    if rest.starts_with("-> ") {
        return Some(parse_arrow_line(parser, indent_level));
    }

    // 7. ConditionLine: ? expr
    if rest.starts_with("? ") {
        return Some(parse_condition_line(parser, indent_level));
    }

    // 8. EffectLine: > effect
    if rest.starts_with("> ") {
        return Some(parse_effect_line(parser, indent_level));
    }

    // 9. ChoiceLine: * or + with label (Text requires at least one character)
    if rest.starts_with("* ") || rest.starts_with("+ ") {
        // Check that there's actual label text after the sigil
        let after_choice_sigil = &rest[2..];
        let label_check = if let Some(arrow_pos) = after_choice_sigil.find(" -> ") {
            Parser::strip_inline_comment(&after_choice_sigil[..arrow_pos]).trim()
        } else {
            Parser::strip_inline_comment(after_choice_sigil).trim()
        };
        if label_check.is_empty() {
            return Some(make_error_node(parser, line_idx, None));
        }
        return Some(parse_choice_line(parser, indent_level));
    }

    // 10. BlockedMessage: ! text
    if rest.starts_with("! ") {
        return Some(parse_blocked_message(parser, indent_level));
    }

    // 11. EntityPresence: [@entity, @entity]
    if rest.starts_with("[@") {
        return Some(parse_entity_presence(parser));
    }

    // 12. LineComment: // text
    if rest.starts_with("// ") || rest == "//" {
        return Some(parse_line_comment(parser));
    }

    // 13. Check for grammar rejections that should produce ErrorNode
    if let Some(error) = check_grammar_rejections(parser, rest, line_idx) {
        return Some(error);
    }

    // 14. Prose: fallback — any non-blank line
    Some(parse_prose(parser))
}

// ── Individual node parsers ──

fn parse_location_heading(parser: &mut Parser) -> ContentNode {
    let line_idx = parser.current_line;
    let text = parser.check_tabs(line_idx);
    let (_, rest) = Parser::measure_indent(&text);
    let after_sigil = &rest[2..]; // skip "# "
    let display_name = Parser::strip_inline_comment(after_sigil).trim().to_string();
    let span = parser.content_line_span(line_idx);
    parser.current_line += 1;

    ContentNode::LocationHeading(LocationHeading { display_name, span })
}

fn parse_sequence_heading(parser: &mut Parser) -> ContentNode {
    let line_idx = parser.current_line;
    let text = parser.check_tabs(line_idx);
    let (_, rest) = Parser::measure_indent(&text);
    let after_sigil = &rest[3..]; // skip "## "
    let display_name = Parser::strip_inline_comment(after_sigil).trim().to_string();
    let span = parser.content_line_span(line_idx);
    parser.current_line += 1;

    ContentNode::SequenceHeading(SequenceHeading { display_name, span })
}

fn parse_phase_heading(parser: &mut Parser) -> ContentNode {
    let line_idx = parser.current_line;
    let text = parser.check_tabs(line_idx);
    let (_, rest) = Parser::measure_indent(&text);
    let after_sigil = &rest[4..]; // skip "### "
    let content = Parser::strip_inline_comment(after_sigil).trim();

    let (display_name, auto) = if content.ends_with("(auto)") {
        (content[..content.len() - 6].trim().to_string(), true)
    } else {
        (content.to_string(), false)
    };

    let span = parser.content_line_span(line_idx);
    parser.current_line += 1;

    ContentNode::PhaseHeading(PhaseHeading { display_name, auto, span })
}

fn parse_section_label(parser: &mut Parser) -> ContentNode {
    let line_idx = parser.current_line;
    let text = parser.check_tabs(line_idx);
    let (_, rest) = Parser::measure_indent(&text);
    let name = rest[3..].trim().to_string(); // skip "== "
    let span = parser.content_line_span(line_idx);
    parser.current_line += 1;

    ContentNode::SectionLabel(SectionLabel { name, span })
}

fn parse_entity_line(parser: &mut Parser, _indent_level: usize) -> Option<ContentNode> {
    let line_idx = parser.current_line;
    let text = parser.check_tabs(line_idx);
    let (_, rest) = Parser::measure_indent(&text);

    // rest starts with @
    let after_at = &rest[1..];

    // Extract entity identifier (lowercase alphanumeric + underscore)
    let id_end = after_at
        .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
        .unwrap_or(after_at.len());

    if id_end == 0 {
        // No valid identifier after @
        return Some(make_error_node(parser, line_idx, None));
    }

    let entity_ref = after_at[..id_end].to_string();

    // Check that the identifier starts with lowercase
    if entity_ref.chars().next().map_or(true, |c| c.is_uppercase()) {
        return Some(make_error_node(parser, line_idx, None));
    }

    let remaining = &after_at[id_end..];

    // EntitySpeech: @name: text
    if remaining.starts_with(": ") || remaining == ":" {
        let speech_text = if remaining.len() > 2 {
            Parser::strip_inline_comment(&remaining[2..]).trim().to_string()
        } else {
            String::new()
        };
        let span = parser.content_line_span(line_idx);
        parser.current_line += 1;
        return Some(ContentNode::EntitySpeech(EntitySpeech {
            entity_ref,
            text: speech_text,
            annotation: None,
            span,
        }));
    }

    // StageDirection: @name text (space followed by action text)
    if remaining.starts_with(' ') {
        let action_text = Parser::strip_inline_comment(&remaining[1..]).trim().to_string();
        let span = parser.content_line_span(line_idx);
        parser.current_line += 1;
        return Some(ContentNode::StageDirection(StageDirection {
            entity_ref,
            text: action_text,
            annotation: None,
            span,
        }));
    }

    // If there's nothing after the name, treat as stage direction with empty text
    if remaining.is_empty() {
        let span = parser.content_line_span(line_idx);
        parser.current_line += 1;
        return Some(ContentNode::StageDirection(StageDirection {
            entity_ref,
            text: String::new(),
            annotation: None,
            span,
        }));
    }

    // Could not parse as entity line
    Some(make_error_node(parser, line_idx, None))
}

fn parse_arrow_line(parser: &mut Parser, indent_level: usize) -> ContentNode {
    let line_idx = parser.current_line;
    let text = parser.check_tabs(line_idx);
    let (_, rest) = Parser::measure_indent(&text);
    let after_arrow = &rest[3..]; // skip "-> "

    // ExitDeclaration: -> direction: Destination
    // Must have `: ` with an identifier before it (not `exit:`)
    if let Some(colon_pos) = after_arrow.find(": ") {
        let before_colon = &after_arrow[..colon_pos];
        // Make sure it's not `exit:` pattern
        if before_colon != "exit" && !before_colon.contains(':') {
            let direction = before_colon.trim().to_string();
            let destination = after_arrow[colon_pos + 2..].trim().to_string();
            let span = parser.content_line_span(line_idx);
            parser.current_line += 1;

            // Collect exit children (indented Condition and BlockedMessage)
            let children = parse_exit_children(parser, indent_level);

            return ContentNode::ExitDeclaration(ExitDeclaration {
                direction,
                destination,
                children,
                annotation: None,
                span,
            });
        }
    }

    // Exit-qualified jump: -> exit:name
    if after_arrow.starts_with("exit:") {
        let target = after_arrow[5..].trim().to_string();
        let span = parser.content_line_span(line_idx);
        parser.current_line += 1;
        return ContentNode::Jump(Jump {
            target,
            is_exit_qualified: true,
            indent_level,
            annotation: None,
            span,
        });
    }

    // Simple jump: -> name
    let target = after_arrow.trim().to_string();
    let span = parser.content_line_span(line_idx);
    parser.current_line += 1;
    ContentNode::Jump(Jump {
        target,
        is_exit_qualified: false,
        indent_level,
        annotation: None,
        span,
    })
}

/// Collect children for an ExitDeclaration (Condition and BlockedMessage nodes).
fn parse_exit_children(parser: &mut Parser, exit_indent: usize) -> Vec<ContentNode> {
    let mut children = Vec::new();

    while !parser.at_end() {
        let line = match parser.peek_line() {
            Some(l) => l,
            None => break,
        };

        // Skip blank lines
        if line.trim().is_empty() {
            parser.current_line += 1;
            continue;
        }

        let processed = parser.check_tabs(parser.current_line);
        let (indent_level, rest) = Parser::measure_indent(&processed);

        // Must be strictly deeper than exit
        if indent_level <= exit_indent {
            break;
        }

        // Only Condition and BlockedMessage attach as exit children
        if rest.starts_with("? ") {
            children.push(parse_condition_line(parser, indent_level));
        } else if rest.starts_with("! ") {
            children.push(parse_blocked_message(parser, indent_level));
        } else {
            // Not a valid exit child — stop collecting children
            break;
        }
    }

    children
}

fn parse_condition_line(parser: &mut Parser, indent_level: usize) -> ContentNode {
    let line_idx = parser.current_line;
    let text = parser.check_tabs(line_idx);
    let (_, rest) = Parser::measure_indent(&text);
    let after_sigil = &rest[2..]; // skip "? "
    let span = parser.content_line_span(line_idx);
    parser.current_line += 1;

    match parse_condition_expr(after_sigil, &span) {
        Some(expr) => ContentNode::Condition(Condition {
            expr,
            indent_level,
            span,
        }),
        None => {
            // URD112: Malformed condition expression
            let line_num = parser.lines[line_idx].line_number;
            let display = truncate_for_display(parser.lines[line_idx].text.trim());
            parser.diagnostics.error(
                "URD112",
                format!("Unrecognised syntax at line {}: '{}'.", line_num, display),
                span.clone(),
            );
            ContentNode::ErrorNode(ErrorNode {
                raw_text: parser.lines[line_idx].text.to_string(),
                attempted_rule: Some("ConditionExpr".to_string()),
                span,
            })
        }
    }
}

/// Parse a condition expression string into a ConditionExpr.
pub(crate) fn parse_condition_expr(expr: &str, span: &Span) -> Option<ConditionExpr> {
    let expr = expr.trim();

    // ExhaustionCheck: identifier.exhausted
    if expr.ends_with(".exhausted") {
        let section = &expr[..expr.len() - 10]; // strip ".exhausted"
        return Some(ConditionExpr::ExhaustionCheck(ExhaustionCheck {
            section_name: section.to_string(),
            annotation: None,
            span: span.clone(),
        }));
    }

    // ContainmentCheck: @entity in container / @entity not in container
    if expr.starts_with('@') {
        let rest = &expr[1..];

        // Find end of entity identifier
        let id_end = rest
            .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
            .unwrap_or(rest.len());
        let entity = &rest[..id_end];
        let after_entity = rest[id_end..].trim();

        if after_entity.starts_with("not in ") {
            let container = after_entity[7..].trim().to_string();
            return Some(ConditionExpr::ContainmentCheck(ContainmentCheck {
                entity_ref: entity.to_string(),
                container_ref: container,
                negated: true,
                annotation: None,
                span: span.clone(),
            }));
        }

        if after_entity.starts_with("in ") {
            let container = after_entity[3..].trim().to_string();
            return Some(ConditionExpr::ContainmentCheck(ContainmentCheck {
                entity_ref: entity.to_string(),
                container_ref: container,
                negated: false,
                annotation: None,
                span: span.clone(),
            }));
        }

        // PropertyComparison: @entity.property op value
        if let Some(dot_pos) = rest[..id_end].find('.').or_else(|| {
            // dot might be after the entity id
            if rest[id_end..].starts_with('.') {
                Some(id_end)
            } else {
                None
            }
        }) {
            let entity_ref;
            let remaining;

            if dot_pos < id_end {
                // dot is within what we thought was the id
                entity_ref = rest[..dot_pos].to_string();
                remaining = &rest[dot_pos + 1..];
            } else {
                // dot is right after the id
                entity_ref = rest[..id_end].to_string();
                remaining = &rest[id_end + 1..];
            }

            // Find the operator: ==, !=, >=, <=, >, <
            let ops = ["==", "!=", ">=", "<=", ">", "<"];
            for op in &ops {
                if let Some(op_pos) = remaining.find(op) {
                    let property = remaining[..op_pos].trim().to_string();
                    let value = remaining[op_pos + op.len()..].trim().to_string();
                    if !property.is_empty() {
                        return Some(ConditionExpr::PropertyComparison(PropertyComparison {
                            entity_ref,
                            property,
                            operator: op.to_string(),
                            value,
                            annotation: None,
                            span: span.clone(),
                        }));
                    }
                }
            }
        }
    }

    None
}

fn parse_or_condition_block(parser: &mut Parser, indent_level: usize) -> ContentNode {
    let start_line = parser.current_line;
    let span_start = parser.content_line_span(start_line);
    parser.current_line += 1; // skip "? any:" line

    let mut conditions = Vec::new();

    while !parser.at_end() {
        let line = match parser.peek_line() {
            Some(l) => l,
            None => break,
        };

        // Blank lines terminate the OrConditionBlock
        if line.trim().is_empty() {
            break;
        }

        let processed = parser.check_tabs(parser.current_line);
        let (line_indent, rest) = Parser::measure_indent(&processed);

        // Must be deeper than the ? any: line
        if line_indent <= indent_level {
            break;
        }

        // Parse as a bare condition expression (without ? prefix)
        let line_span = parser.line_span(parser.current_line);
        if let Some(expr) = parse_condition_expr(rest, &line_span) {
            conditions.push(expr);
        }
        parser.current_line += 1;
    }

    let span = if parser.current_line > start_line + 1 {
        parser.content_span_lines(start_line, parser.current_line.saturating_sub(1))
    } else {
        span_start
    };

    ContentNode::OrConditionBlock(OrConditionBlock {
        conditions,
        indent_level,
        span,
    })
}

fn parse_effect_line(parser: &mut Parser, indent_level: usize) -> ContentNode {
    let line_idx = parser.current_line;
    let text = parser.check_tabs(line_idx);
    let (_, rest) = Parser::measure_indent(&text);
    let after_sigil = &rest[2..]; // skip "> "
    let span = parser.content_line_span(line_idx);
    parser.current_line += 1;

    let effect_type = parse_effect_type(after_sigil);

    ContentNode::Effect(Effect {
        effect_type,
        indent_level,
        annotation: None,
        span,
    })
}

/// Parse the effect type from the text after `> `.
fn parse_effect_type(text: &str) -> EffectType {
    let text = text.trim();

    // Move effect: move @entity -> container
    if text.starts_with("move ") {
        let rest = &text[5..];
        if rest.starts_with('@') {
            let after_at = &rest[1..];
            if let Some(arrow_pos) = after_at.find(" -> ") {
                let entity = after_at[..arrow_pos].trim().to_string();
                let dest = after_at[arrow_pos + 4..].trim().to_string();
                return EffectType::Move {
                    entity_ref: entity,
                    destination_ref: dest,
                };
            }
        }
    }

    // Reveal effect: reveal @entity.prop
    if text.starts_with("reveal ") {
        let target = text[7..].trim().to_string();
        return EffectType::Reveal { target_prop: target };
    }

    // Destroy effect: destroy @entity
    if text.starts_with("destroy ") {
        let rest = &text[8..].trim();
        let entity = if rest.starts_with('@') {
            rest[1..].to_string()
        } else {
            rest.to_string()
        };
        return EffectType::Destroy { entity_ref: entity };
    }

    // Set effect: @entity.prop = value or @entity.prop + N or @entity.prop - N
    // Look for operator: =, +, -
    let ops = [" = ", " + ", " - "];
    for op_str in &ops {
        if let Some(op_pos) = text.find(op_str) {
            let target_prop = text[..op_pos].trim().to_string();
            let operator = op_str.trim().to_string();
            let value_expr = text[op_pos + op_str.len()..].trim().to_string();
            return EffectType::Set {
                target_prop,
                operator,
                value_expr,
            };
        }
    }

    // Fallback: treat as set with = operator
    EffectType::Set {
        target_prop: text.to_string(),
        operator: "=".to_string(),
        value_expr: String::new(),
    }
}

fn parse_choice_line(parser: &mut Parser, indent_level: usize) -> ContentNode {
    let line_idx = parser.current_line;
    let text = parser.check_tabs(line_idx);
    let (_, rest) = Parser::measure_indent(&text);

    let sticky = rest.starts_with('+');
    let after_sigil = &rest[2..]; // skip "* " or "+ "

    // Check for target: label -> target
    let (label, target, target_type) = if let Some(arrow_pos) = after_sigil.find(" -> ") {
        let label_text = Parser::strip_inline_comment(&after_sigil[..arrow_pos]).trim().to_string();
        let target_text = after_sigil[arrow_pos + 4..].trim();

        if target_text.starts_with('@') {
            // Entity target
            (label_text, Some(target_text[1..].to_string()), None)
        } else if target_text.starts_with("any ") {
            // Type target
            let type_name = target_text[4..].trim().to_string();
            (label_text, None, Some(type_name))
        } else {
            // Section/exit target
            (label_text, Some(target_text.to_string()), None)
        }
    } else {
        let label_text = Parser::strip_inline_comment(after_sigil).trim().to_string();
        (label_text, None, None)
    };

    let span = parser.content_line_span(line_idx);
    parser.current_line += 1;

    // Parse nested content (lines indented deeper than this choice)
    let child_indent = indent_level + 1;
    let content = parse_content(parser, child_indent);

    ContentNode::Choice(Choice {
        sticky,
        label,
        target,
        target_type,
        content,
        indent_level,
        annotation: None,
        span,
    })
}

fn parse_blocked_message(parser: &mut Parser, indent_level: usize) -> ContentNode {
    let line_idx = parser.current_line;
    let text = parser.check_tabs(line_idx);
    let (_, rest) = Parser::measure_indent(&text);
    let msg_text = Parser::strip_inline_comment(&rest[2..]).trim().to_string(); // skip "! "
    let span = parser.content_line_span(line_idx);
    parser.current_line += 1;

    ContentNode::BlockedMessage(BlockedMessage {
        text: msg_text,
        indent_level,
        span,
    })
}

fn parse_entity_presence(parser: &mut Parser) -> ContentNode {
    let line_idx = parser.current_line;
    let text = parser.check_tabs(line_idx);
    let (_, rest) = Parser::measure_indent(&text);
    let span = parser.content_line_span(line_idx);
    parser.current_line += 1;

    // Extract entities from [@entity1, @entity2]
    let inner = if rest.starts_with('[') && rest.ends_with(']') {
        &rest[1..rest.len() - 1]
    } else {
        rest
    };

    let entity_refs: Vec<String> = inner
        .split(',')
        .map(|s| {
            let s = s.trim();
            if s.starts_with('@') {
                s[1..].to_string()
            } else {
                s.to_string()
            }
        })
        .filter(|s| !s.is_empty())
        .collect();

    let annotations = entity_refs.iter().map(|_| None).collect();

    ContentNode::EntityPresence(EntityPresence {
        entity_refs,
        annotations,
        span,
    })
}

fn parse_line_comment(parser: &mut Parser) -> ContentNode {
    let line_idx = parser.current_line;
    let text = parser.check_tabs(line_idx);
    let (_, rest) = Parser::measure_indent(&text);
    let comment_text = if rest.len() > 3 {
        rest[3..].to_string() // skip "// "
    } else {
        String::new() // bare "//"
    };
    let span = parser.content_line_span(line_idx);
    parser.current_line += 1;

    ContentNode::Comment(Comment {
        text: comment_text,
        span,
    })
}

fn parse_prose(parser: &mut Parser) -> ContentNode {
    let line_idx = parser.current_line;
    let text = parser.check_tabs(line_idx);
    let (_, rest) = Parser::measure_indent(&text);
    let prose_text = Parser::strip_inline_comment(rest).trim().to_string();
    let span = parser.content_line_span(line_idx);
    parser.current_line += 1;

    ContentNode::Prose(Prose {
        text: prose_text,
        span,
    })
}

fn parse_rule_block(parser: &mut Parser, _indent_level: usize) -> ContentNode {
    let start_line = parser.current_line;
    let text = parser.check_tabs(start_line);
    let (rule_indent, rest) = Parser::measure_indent(&text);

    // Extract rule name from "rule name:"
    let name = rest[5..rest.len() - 1].trim().to_string(); // skip "rule " and trailing ":"

    parser.current_line += 1;

    let mut actor = String::new();
    let mut trigger = String::new();
    let mut select: Option<SelectClause> = None;
    let mut where_clauses: Vec<ConditionExpr> = Vec::new();
    let mut effects: Vec<Effect> = Vec::new();

    // Parse body lines (strictly deeper than rule header)
    while !parser.at_end() {
        let line = match parser.peek_line() {
            Some(l) => l,
            None => break,
        };

        if line.trim().is_empty() {
            break; // blank line ends rule block
        }

        let processed = parser.check_tabs(parser.current_line);
        let (body_indent, body_rest) = Parser::measure_indent(&processed);

        if body_indent <= rule_indent {
            break;
        }

        let body_span = parser.line_span(parser.current_line);

        // Actor line: actor: @entity trigger
        if body_rest.starts_with("actor: ") || body_rest.starts_with("actor:") {
            let after = body_rest.strip_prefix("actor:").unwrap().trim();
            if after.starts_with('@') {
                let after_at = &after[1..];
                let parts: Vec<&str> = after_at.splitn(2, ' ').collect();
                actor = parts[0].to_string();
                if parts.len() > 1 {
                    trigger = parts[1].trim().to_string();
                }
            }
            parser.current_line += 1;
            continue;
        }

        // Select line: selects variable from [@a, @b]
        if body_rest.starts_with("selects ") {
            if let Some(from_pos) = body_rest.find(" from ") {
                let variable = body_rest[8..from_pos].trim().to_string();
                let from_text = &body_rest[from_pos + 6..];
                let entity_refs = parse_entity_ref_list(from_text);

                // Collect where clauses that follow (select-scoped)
                parser.current_line += 1;
                let mut select_where = Vec::new();

                while !parser.at_end() {
                    let wl = match parser.peek_line() {
                        Some(l) => l,
                        None => break,
                    };
                    if wl.trim().is_empty() {
                        break;
                    }
                    let wp = parser.check_tabs(parser.current_line);
                    let (wi, wr) = Parser::measure_indent(&wp);
                    if wi <= rule_indent {
                        break;
                    }
                    if wr.starts_with("where ") {
                        let ws = parser.line_span(parser.current_line);
                        if let Some(expr) = parse_condition_expr(&wr[6..], &ws) {
                            select_where.push(expr);
                        }
                        parser.current_line += 1;
                    } else {
                        break;
                    }
                }

                select = Some(SelectClause {
                    variable,
                    entity_refs,
                    where_clauses: select_where,
                    span: body_span,
                });
                continue;
            }
        }

        // Where line: where condition
        if body_rest.starts_with("where ") {
            if let Some(expr) = parse_condition_expr(&body_rest[6..], &body_span) {
                where_clauses.push(expr);
            }
            parser.current_line += 1;
            continue;
        }

        // Effect line: > effect
        if body_rest.starts_with("> ") {
            let effect_type = parse_effect_type(&body_rest[2..]);
            effects.push(Effect {
                effect_type,
                indent_level: body_indent,
                annotation: None,
                span: body_span,
            });
            parser.current_line += 1;
            continue;
        }

        // Unknown line in rule body — skip
        parser.current_line += 1;
    }

    let span = parser.content_span_lines(start_line, parser.current_line.saturating_sub(1).max(start_line));

    ContentNode::RuleBlock(RuleBlock {
        name,
        actor,
        trigger,
        select,
        where_clauses,
        effects,
        span,
    })
}

/// Parse an entity reference list like `[@a, @b, @c]`.
fn parse_entity_ref_list(text: &str) -> Vec<String> {
    let text = text.trim();
    let inner = if text.starts_with('[') && text.ends_with(']') {
        &text[1..text.len() - 1]
    } else {
        text
    };

    inner
        .split(',')
        .map(|s| {
            let s = s.trim();
            if s.starts_with('@') {
                s[1..].to_string()
            } else {
                s.to_string()
            }
        })
        .filter(|s| !s.is_empty())
        .collect()
}

/// Check for grammar rejections that should produce an ErrorNode.
fn check_grammar_rejections(parser: &mut Parser, rest: &str, line_idx: usize) -> Option<ContentNode> {
    // Missing space after heading sigil: #Heading, ##Heading, ###Heading
    if (rest.starts_with('#') && !rest.starts_with("# ") && !rest.starts_with("## ") && !rest.starts_with("### "))
        || (rest.starts_with("?") && !rest.starts_with("? ") && !rest.starts_with("? any:"))
    {
        return Some(make_error_node(parser, line_idx, None));
    }

    None
}

/// Create an ErrorNode and emit URD112 diagnostic.
fn make_error_node(parser: &mut Parser, line_idx: usize, attempted_rule: Option<&str>) -> ContentNode {
    let raw_text = parser.lines[line_idx].text.to_string();
    let line_num = parser.lines[line_idx].line_number;
    let display = truncate_for_display(raw_text.trim());
    let span = parser.content_line_span(line_idx);

    parser.diagnostics.error(
        "URD112",
        format!("Unrecognised syntax at line {}: '{}'.", line_num, display),
        span.clone(),
    );

    parser.current_line += 1;

    ContentNode::ErrorNode(ErrorNode {
        raw_text,
        attempted_rule: attempted_rule.map(|s| s.to_string()),
        span,
    })
}

/// Truncate text for display in diagnostics.
fn truncate_for_display(text: &str) -> String {
    if text.len() > 60 {
        format!("{}...", &text[..57])
    } else {
        text.to_string()
    }
}
