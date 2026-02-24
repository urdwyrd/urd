/// Semantic diff engine: compare two compiled worlds at the structural level.
///
/// Produces a typed change report over six categories: entity, location/exit,
/// dialogue (section/choice), property dependency, rule, and reachability.
///
/// The diff operates on `DiffSnapshot` values — normalised, comparable
/// representations of compiled output. Snapshots can be built from a live
/// `CompilationResult` or loaded from a `.urd.snapshot.json` file.

use std::collections::BTreeSet;
use indexmap::IndexMap;
use serde_json::Value as Json;

use crate::CompilationResult;
use crate::facts::{JumpTarget, PropertyKey};

// ── Snapshot structs ──

#[derive(Debug, Clone, PartialEq)]
pub struct EntitySnapshot {
    pub entity_type: String,
    pub properties: IndexMap<String, String>,
    pub container: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocationSnapshot {
    pub display_name: String,
    pub entity_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExitSnapshot {
    pub from: String,
    pub to: String,
    pub is_conditional: bool,
    pub guard_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SectionSnapshot {
    pub choice_ids: Vec<String>,
    pub jump_targets: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChoiceSnapshot {
    pub label: String,
    pub sticky: bool,
    pub condition_count: usize,
    pub effect_count: usize,
    pub jump_targets: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleSnapshot {
    pub condition_count: usize,
    pub effect_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PropertySnapshot {
    pub read_count: usize,
    pub write_count: usize,
    pub orphaned: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DiagnosticKey {
    pub code: String,
    pub target_id: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiffSnapshot {
    pub entities: IndexMap<String, EntitySnapshot>,
    pub locations: IndexMap<String, LocationSnapshot>,
    pub exits: IndexMap<String, ExitSnapshot>,
    pub sections: IndexMap<String, SectionSnapshot>,
    pub choices: IndexMap<String, ChoiceSnapshot>,
    pub rules: IndexMap<String, RuleSnapshot>,
    pub properties: IndexMap<String, PropertySnapshot>,
    pub diagnostic_keys: BTreeSet<DiagnosticKey>,
}

#[derive(Debug)]
pub enum DiffError {
    UnsupportedSnapshotVersion,
    ParseError(String),
}

impl std::fmt::Display for DiffError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiffError::UnsupportedSnapshotVersion => {
                write!(f, "Unsupported snapshot version (expected \"1\")")
            }
            DiffError::ParseError(msg) => write!(f, "Snapshot parse error: {}", msg),
        }
    }
}

// ── DiffSnapshot construction ──

impl DiffSnapshot {
    /// Build a DiffSnapshot from a live compilation result.
    pub fn from_compilation(result: &CompilationResult) -> Self {
        let mut entities = IndexMap::new();
        let mut locations = IndexMap::new();
        let mut exits = IndexMap::new();
        let mut sections: IndexMap<String, SectionSnapshot> = IndexMap::new();
        let mut choices = IndexMap::new();
        let mut rules = IndexMap::new();
        let mut properties = IndexMap::new();
        let mut diagnostic_keys = BTreeSet::new();

        // Build a reverse map: entity_id (without @) → location_id.
        let mut entity_container: IndexMap<String, String> = IndexMap::new();

        // Parse world JSON for entities and locations.
        if let Some(ref world_json) = result.world {
            if let Ok(world) = serde_json::from_str::<Json>(world_json) {
                // Entities
                if let Some(ents) = world.get("entities").and_then(|v| v.as_object()) {
                    for (id, val) in ents {
                        let entity_type = val
                            .get("type")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let mut props = IndexMap::new();
                        if let Some(p) = val.get("properties").and_then(|v| v.as_object()) {
                            for (k, v) in p {
                                props.insert(k.clone(), json_value_to_string(v));
                            }
                        }
                        entities.insert(
                            format!("@{}", id),
                            EntitySnapshot {
                                entity_type,
                                properties: props,
                                container: None, // filled below from locations
                            },
                        );
                    }
                }

                // Locations
                if let Some(locs) = world.get("locations").and_then(|v| v.as_object()) {
                    for (id, val) in locs {
                        let display_name = val
                            .get("description")
                            .and_then(|v| v.as_str())
                            .unwrap_or(id)
                            .to_string();
                        let mut entity_ids = Vec::new();
                        if let Some(contains) =
                            val.get("contains").and_then(|v| v.as_array())
                        {
                            for e in contains {
                                if let Some(eid) = e.as_str() {
                                    let full_id = format!("@{}", eid);
                                    entity_ids.push(full_id.clone());
                                    entity_container.insert(full_id, id.clone());
                                }
                            }
                        }
                        locations.insert(
                            id.clone(),
                            LocationSnapshot {
                                display_name,
                                entity_ids,
                            },
                        );
                    }
                }
            }
        }

        // Apply container info to entities.
        for (id, snap) in entities.iter_mut() {
            if let Some(loc) = entity_container.get(id) {
                snap.container = Some(loc.clone());
            }
        }

        // Extract from FactSet.
        if let Some(ref fs) = result.fact_set {
            // Exits
            for edge in fs.exits() {
                exits.insert(
                    edge.exit_id(),
                    ExitSnapshot {
                        from: edge.from_location.clone(),
                        to: edge.to_location.clone(),
                        is_conditional: edge.is_conditional,
                        guard_count: edge.guard_reads.len(),
                    },
                );
            }

            // Collect jump indices owned by choices.
            let mut choice_owned_jumps: BTreeSet<usize> = BTreeSet::new();
            for choice in fs.choices() {
                for &idx in &choice.jump_indices {
                    choice_owned_jumps.insert(idx);
                }
            }

            // Choices
            for choice in fs.choices() {
                let jump_targets: Vec<String> = choice
                    .jump_indices
                    .iter()
                    .filter_map(|&idx| fs.jumps().get(idx))
                    .map(|j| jump_target_string(&j.target))
                    .collect();

                choices.insert(
                    choice.choice_id.clone(),
                    ChoiceSnapshot {
                        label: choice.label.clone(),
                        sticky: choice.sticky,
                        condition_count: choice.condition_reads.len(),
                        effect_count: choice.effect_writes.len(),
                        jump_targets,
                    },
                );

                // Ensure the section exists in the sections map.
                sections
                    .entry(choice.section.clone())
                    .or_insert_with(|| SectionSnapshot {
                        choice_ids: Vec::new(),
                        jump_targets: Vec::new(),
                    })
                    .choice_ids
                    .push(choice.choice_id.clone());
            }

            // Section-level jumps (not owned by any choice).
            for (idx, jump) in fs.jumps().iter().enumerate() {
                if !choice_owned_jumps.contains(&idx) {
                    sections
                        .entry(jump.from_section.clone())
                        .or_insert_with(|| SectionSnapshot {
                            choice_ids: Vec::new(),
                            jump_targets: Vec::new(),
                        })
                        .jump_targets
                        .push(jump_target_string(&jump.target));
                }
            }

            // Rules
            for rule in fs.rules() {
                rules.insert(
                    rule.rule_id.clone(),
                    RuleSnapshot {
                        condition_count: rule.condition_reads.len(),
                        effect_count: rule.effect_writes.len(),
                    },
                );
            }
        }

        // Properties from PropertyDependencyIndex.
        if let Some(ref idx) = result.property_index {
            let read_never_written: BTreeSet<(String, String)> = idx
                .read_but_never_written()
                .into_iter()
                .map(|k| (k.entity_type.clone(), k.property.clone()))
                .collect();
            let written_never_read: BTreeSet<(String, String)> = idx
                .written_but_never_read()
                .into_iter()
                .map(|k| (k.entity_type.clone(), k.property.clone()))
                .collect();

            // Collect all keys.
            let mut all_keys: Vec<PropertyKey> = Vec::new();
            for k in idx.read_properties() {
                all_keys.push(k.clone());
            }
            for k in idx.written_properties() {
                if !all_keys.iter().any(|existing| {
                    existing.entity_type == k.entity_type && existing.property == k.property
                }) {
                    all_keys.push(k.clone());
                }
            }
            all_keys.sort_by(|a, b| {
                (&a.entity_type, &a.property).cmp(&(&b.entity_type, &b.property))
            });

            for key in &all_keys {
                let prop_key = format!("{}.{}", key.entity_type, key.property);
                let pair = (key.entity_type.clone(), key.property.clone());
                let orphaned = if read_never_written.contains(&pair) {
                    Some("read_never_written".to_string())
                } else if written_never_read.contains(&pair) {
                    Some("written_never_read".to_string())
                } else {
                    None
                };

                properties.insert(
                    prop_key,
                    PropertySnapshot {
                        read_count: idx.reads_of(key).len(),
                        write_count: idx.writes_of(key).len(),
                        orphaned,
                    },
                );
            }
        }

        // Diagnostic keys (URD430, URD432).
        for diag in result.diagnostics.all() {
            if diag.code == "URD430" {
                if let Some(target) = extract_urd430_target(&diag.message) {
                    diagnostic_keys.insert(DiagnosticKey {
                        code: "URD430".to_string(),
                        target_id: target,
                    });
                }
            } else if diag.code == "URD432" {
                if let Some(target) = extract_urd432_target(&diag.message) {
                    diagnostic_keys.insert(DiagnosticKey {
                        code: "URD432".to_string(),
                        target_id: target,
                    });
                }
            }
        }

        DiffSnapshot {
            entities,
            locations,
            exits,
            sections,
            choices,
            rules,
            properties,
            diagnostic_keys,
        }
    }
}

/// Convert a JumpTarget to a comparable string.
fn jump_target_string(target: &JumpTarget) -> String {
    match target {
        JumpTarget::Section(id) => id.clone(),
        JumpTarget::Exit(id) => format!("exit:{}", id),
        JumpTarget::End => "__end__".to_string(),
    }
}

/// Convert a JSON value to a display string for property comparison.
fn json_value_to_string(v: &Json) -> String {
    match v {
        Json::String(s) => s.clone(),
        Json::Bool(b) => b.to_string(),
        Json::Number(n) => n.to_string(),
        Json::Null => "null".to_string(),
        other => other.to_string(),
    }
}

/// Extract location slug from URD430 message.
/// Pattern: "Location '...' is unreachable"
pub fn extract_urd430_target(message: &str) -> Option<String> {
    let start = message.find("Location '")? + "Location '".len();
    let rest = &message[start..];
    let end = rest.find('\'')?;
    Some(rest[..end].to_string())
}

/// Extract section compiled_id from URD432 message.
/// Pattern: "Choice in section '...'"
pub fn extract_urd432_target(message: &str) -> Option<String> {
    let start = message.find("Choice in section '")? + "Choice in section '".len();
    let rest = &message[start..];
    let end = rest.find('\'')?;
    Some(rest[..end].to_string())
}

// ── JSON serialisation ──

impl DiffSnapshot {
    /// Serialise to a snapshot JSON value.
    pub fn to_json(&self, world_name: &str) -> serde_json::Value {
        let entities: serde_json::Map<String, Json> = self
            .entities
            .iter()
            .map(|(id, snap)| {
                let props: serde_json::Map<String, Json> = snap
                    .properties
                    .iter()
                    .map(|(k, v)| (k.clone(), Json::String(v.clone())))
                    .collect();
                (
                    id.clone(),
                    serde_json::json!({
                        "type": snap.entity_type,
                        "properties": Json::Object(props),
                        "container": snap.container,
                    }),
                )
            })
            .collect();

        let locations: serde_json::Map<String, Json> = self
            .locations
            .iter()
            .map(|(id, snap)| {
                (
                    id.clone(),
                    serde_json::json!({
                        "display_name": snap.display_name,
                        "entity_ids": snap.entity_ids,
                    }),
                )
            })
            .collect();

        let exits: serde_json::Map<String, Json> = self
            .exits
            .iter()
            .map(|(id, snap)| {
                (
                    id.clone(),
                    serde_json::json!({
                        "from": snap.from,
                        "to": snap.to,
                        "is_conditional": snap.is_conditional,
                        "guard_count": snap.guard_count,
                    }),
                )
            })
            .collect();

        let sections: serde_json::Map<String, Json> = self
            .sections
            .iter()
            .map(|(id, snap)| {
                (
                    id.clone(),
                    serde_json::json!({
                        "choice_ids": snap.choice_ids,
                        "jump_targets": snap.jump_targets,
                    }),
                )
            })
            .collect();

        let choices: serde_json::Map<String, Json> = self
            .choices
            .iter()
            .map(|(id, snap)| {
                (
                    id.clone(),
                    serde_json::json!({
                        "label": snap.label,
                        "sticky": snap.sticky,
                        "condition_count": snap.condition_count,
                        "effect_count": snap.effect_count,
                        "jump_targets": snap.jump_targets,
                    }),
                )
            })
            .collect();

        let rules: serde_json::Map<String, Json> = self
            .rules
            .iter()
            .map(|(id, snap)| {
                (
                    id.clone(),
                    serde_json::json!({
                        "condition_count": snap.condition_count,
                        "effect_count": snap.effect_count,
                    }),
                )
            })
            .collect();

        let properties: serde_json::Map<String, Json> = self
            .properties
            .iter()
            .map(|(id, snap)| {
                (
                    id.clone(),
                    serde_json::json!({
                        "read_count": snap.read_count,
                        "write_count": snap.write_count,
                        "orphaned": snap.orphaned,
                    }),
                )
            })
            .collect();

        let diag_keys: Vec<Json> = self
            .diagnostic_keys
            .iter()
            .map(|dk| {
                serde_json::json!({
                    "code": dk.code,
                    "target_id": dk.target_id,
                })
            })
            .collect();

        serde_json::json!({
            "urd_snapshot": "1",
            "world_name": world_name,
            "entities": Json::Object(entities),
            "locations": Json::Object(locations),
            "exits": Json::Object(exits),
            "sections": Json::Object(sections),
            "choices": Json::Object(choices),
            "rules": Json::Object(rules),
            "properties": Json::Object(properties),
            "diagnostic_keys": diag_keys,
        })
    }

    /// Parse a DiffSnapshot from a snapshot JSON string.
    pub fn from_json(json_str: &str) -> Result<Self, DiffError> {
        let root: Json = serde_json::from_str(json_str)
            .map_err(|e| DiffError::ParseError(e.to_string()))?;

        // Version check.
        let version = root
            .get("urd_snapshot")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DiffError::ParseError("Missing urd_snapshot field".to_string()))?;
        if version != "1" {
            return Err(DiffError::UnsupportedSnapshotVersion);
        }

        let entities = parse_entities(&root)?;
        let locations = parse_locations(&root)?;
        let exits = parse_exits(&root)?;
        let sections = parse_sections(&root)?;
        let choices = parse_choices(&root)?;
        let rules = parse_rules(&root)?;
        let properties = parse_properties(&root)?;
        let diagnostic_keys = parse_diagnostic_keys(&root)?;

        Ok(DiffSnapshot {
            entities,
            locations,
            exits,
            sections,
            choices,
            rules,
            properties,
            diagnostic_keys,
        })
    }
}

// ── JSON parsing helpers ──

fn parse_entities(root: &Json) -> Result<IndexMap<String, EntitySnapshot>, DiffError> {
    let mut map = IndexMap::new();
    if let Some(obj) = root.get("entities").and_then(|v| v.as_object()) {
        for (id, val) in obj {
            let entity_type = val
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let mut properties = IndexMap::new();
            if let Some(p) = val.get("properties").and_then(|v| v.as_object()) {
                for (k, v) in p {
                    properties.insert(k.clone(), json_value_to_string(v));
                }
            }
            let container = val
                .get("container")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            map.insert(
                id.clone(),
                EntitySnapshot {
                    entity_type,
                    properties,
                    container,
                },
            );
        }
    }
    Ok(map)
}

fn parse_locations(root: &Json) -> Result<IndexMap<String, LocationSnapshot>, DiffError> {
    let mut map = IndexMap::new();
    if let Some(obj) = root.get("locations").and_then(|v| v.as_object()) {
        for (id, val) in obj {
            let display_name = val
                .get("display_name")
                .and_then(|v| v.as_str())
                .unwrap_or(id)
                .to_string();
            let mut entity_ids = Vec::new();
            if let Some(arr) = val.get("entity_ids").and_then(|v| v.as_array()) {
                for e in arr {
                    if let Some(s) = e.as_str() {
                        entity_ids.push(s.to_string());
                    }
                }
            }
            map.insert(
                id.clone(),
                LocationSnapshot {
                    display_name,
                    entity_ids,
                },
            );
        }
    }
    Ok(map)
}

fn parse_exits(root: &Json) -> Result<IndexMap<String, ExitSnapshot>, DiffError> {
    let mut map = IndexMap::new();
    if let Some(obj) = root.get("exits").and_then(|v| v.as_object()) {
        for (id, val) in obj {
            map.insert(
                id.clone(),
                ExitSnapshot {
                    from: val
                        .get("from")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    to: val
                        .get("to")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    is_conditional: val
                        .get("is_conditional")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false),
                    guard_count: val
                        .get("guard_count")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as usize,
                },
            );
        }
    }
    Ok(map)
}

fn parse_sections(root: &Json) -> Result<IndexMap<String, SectionSnapshot>, DiffError> {
    let mut map = IndexMap::new();
    if let Some(obj) = root.get("sections").and_then(|v| v.as_object()) {
        for (id, val) in obj {
            let mut choice_ids = Vec::new();
            if let Some(arr) = val.get("choice_ids").and_then(|v| v.as_array()) {
                for e in arr {
                    if let Some(s) = e.as_str() {
                        choice_ids.push(s.to_string());
                    }
                }
            }
            let mut jump_targets = Vec::new();
            if let Some(arr) = val.get("jump_targets").and_then(|v| v.as_array()) {
                for e in arr {
                    if let Some(s) = e.as_str() {
                        jump_targets.push(s.to_string());
                    }
                }
            }
            map.insert(
                id.clone(),
                SectionSnapshot {
                    choice_ids,
                    jump_targets,
                },
            );
        }
    }
    Ok(map)
}

fn parse_choices(root: &Json) -> Result<IndexMap<String, ChoiceSnapshot>, DiffError> {
    let mut map = IndexMap::new();
    if let Some(obj) = root.get("choices").and_then(|v| v.as_object()) {
        for (id, val) in obj {
            let mut jump_targets = Vec::new();
            if let Some(arr) = val.get("jump_targets").and_then(|v| v.as_array()) {
                for e in arr {
                    if let Some(s) = e.as_str() {
                        jump_targets.push(s.to_string());
                    }
                }
            }
            map.insert(
                id.clone(),
                ChoiceSnapshot {
                    label: val
                        .get("label")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    sticky: val
                        .get("sticky")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false),
                    condition_count: val
                        .get("condition_count")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as usize,
                    effect_count: val
                        .get("effect_count")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as usize,
                    jump_targets,
                },
            );
        }
    }
    Ok(map)
}

fn parse_rules(root: &Json) -> Result<IndexMap<String, RuleSnapshot>, DiffError> {
    let mut map = IndexMap::new();
    if let Some(obj) = root.get("rules").and_then(|v| v.as_object()) {
        for (id, val) in obj {
            map.insert(
                id.clone(),
                RuleSnapshot {
                    condition_count: val
                        .get("condition_count")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as usize,
                    effect_count: val
                        .get("effect_count")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as usize,
                },
            );
        }
    }
    Ok(map)
}

fn parse_properties(root: &Json) -> Result<IndexMap<String, PropertySnapshot>, DiffError> {
    let mut map = IndexMap::new();
    if let Some(obj) = root.get("properties").and_then(|v| v.as_object()) {
        for (id, val) in obj {
            let orphaned = val
                .get("orphaned")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            map.insert(
                id.clone(),
                PropertySnapshot {
                    read_count: val
                        .get("read_count")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as usize,
                    write_count: val
                        .get("write_count")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as usize,
                    orphaned,
                },
            );
        }
    }
    Ok(map)
}

fn parse_diagnostic_keys(root: &Json) -> Result<BTreeSet<DiagnosticKey>, DiffError> {
    let mut set = BTreeSet::new();
    if let Some(arr) = root.get("diagnostic_keys").and_then(|v| v.as_array()) {
        for val in arr {
            let code = val
                .get("code")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let target_id = val
                .get("target_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            set.insert(DiagnosticKey { code, target_id });
        }
    }
    Ok(set)
}

// ── Diff engine ──

/// A single change entry in a diff report.
#[derive(Debug, Clone)]
pub struct ChangeEntry {
    pub category: String,
    pub kind: String,
    pub id: String,
    pub detail: Json,
}

impl ChangeEntry {
    fn new(category: &str, kind: &str, id: &str, detail: Json) -> Self {
        ChangeEntry {
            category: category.to_string(),
            kind: kind.to_string(),
            id: id.to_string(),
            detail,
        }
    }
}

/// The result of comparing two DiffSnapshots.
#[derive(Debug)]
pub struct DiffReport {
    pub changes: Vec<ChangeEntry>,
}

/// Category ordering for deterministic output.
fn category_order(cat: &str) -> usize {
    match cat {
        "entity" => 0,
        "location" => 1,
        "exit" => 2,
        "section" => 3,
        "choice" => 4,
        "rule" => 5,
        "property_dependency" => 6,
        "reachability" => 7,
        _ => 8,
    }
}

/// Kind ordering within a category.
fn kind_order(kind: &str) -> usize {
    match kind {
        "added" => 0,
        "removed" => 1,
        _ => 2, // all "changed" variants
    }
}

/// Compare two DiffSnapshots and produce a DiffReport.
pub fn diff(a: &DiffSnapshot, b: &DiffSnapshot) -> DiffReport {
    let mut changes = Vec::new();

    // 1. Entity changes
    diff_maps(&a.entities, &b.entities, "entity", &mut changes, compare_entity);

    // 2. Location changes
    diff_maps(&a.locations, &b.locations, "location", &mut changes, compare_location);

    // 3. Exit changes
    diff_maps(&a.exits, &b.exits, "exit", &mut changes, compare_exit);

    // 4. Section changes
    diff_maps(&a.sections, &b.sections, "section", &mut changes, compare_section);

    // 5. Choice changes
    diff_maps(&a.choices, &b.choices, "choice", &mut changes, compare_choice);

    // 6. Rule changes
    diff_maps(&a.rules, &b.rules, "rule", &mut changes, compare_rule);

    // 7. Property dependency changes
    diff_maps(
        &a.properties,
        &b.properties,
        "property_dependency",
        &mut changes,
        compare_property,
    );

    // 8. Reachability changes (from diagnostic keys)
    diff_diagnostic_keys(&a.diagnostic_keys, &b.diagnostic_keys, &mut changes);

    // Sort deterministically.
    changes.sort_by(|a, b| {
        category_order(&a.category)
            .cmp(&category_order(&b.category))
            .then(kind_order(&a.kind).cmp(&kind_order(&b.kind)))
            .then(a.id.cmp(&b.id))
    });

    DiffReport { changes }
}

/// Generic keyed map comparison.
fn diff_maps<V>(
    a: &IndexMap<String, V>,
    b: &IndexMap<String, V>,
    category: &str,
    changes: &mut Vec<ChangeEntry>,
    compare: fn(&str, &str, &V, &V) -> Vec<ChangeEntry>,
) {
    // Removed: in A but not in B.
    for id in a.keys() {
        if !b.contains_key(id) {
            changes.push(ChangeEntry::new(category, "removed", id, serde_json::json!({})));
        }
    }

    // Added: in B but not in A.
    for id in b.keys() {
        if !a.contains_key(id) {
            changes.push(ChangeEntry::new(category, "added", id, serde_json::json!({})));
        }
    }

    // Modified: in both, delegate to comparator.
    for (id, val_a) in a {
        if let Some(val_b) = b.get(id) {
            changes.extend(compare(category, id, val_a, val_b));
        }
    }
}

// ── Per-category comparators ──

fn compare_entity(
    _category: &str,
    id: &str,
    a: &EntitySnapshot,
    b: &EntitySnapshot,
) -> Vec<ChangeEntry> {
    let mut changes = Vec::new();
    if a.entity_type != b.entity_type {
        changes.push(ChangeEntry::new(
            "entity",
            "type_changed",
            id,
            serde_json::json!({
                "before": a.entity_type,
                "after": b.entity_type,
            }),
        ));
    }
    if a.properties != b.properties {
        changes.push(ChangeEntry::new(
            "entity",
            "default_changed",
            id,
            serde_json::json!({
                "before": a.properties.iter().map(|(k, v)| (k.clone(), Json::String(v.clone()))).collect::<serde_json::Map<String, Json>>(),
                "after": b.properties.iter().map(|(k, v)| (k.clone(), Json::String(v.clone()))).collect::<serde_json::Map<String, Json>>(),
            }),
        ));
    }
    if a.container != b.container {
        changes.push(ChangeEntry::new(
            "entity",
            "container_changed",
            id,
            serde_json::json!({
                "before": a.container,
                "after": b.container,
            }),
        ));
    }
    changes
}

fn compare_location(
    _category: &str,
    _id: &str,
    _a: &LocationSnapshot,
    _b: &LocationSnapshot,
) -> Vec<ChangeEntry> {
    // Location-level attribute changes are captured via entity container changes
    // and exit changes. No additional comparisons needed here.
    Vec::new()
}

fn compare_exit(
    _category: &str,
    id: &str,
    a: &ExitSnapshot,
    b: &ExitSnapshot,
) -> Vec<ChangeEntry> {
    let mut changes = Vec::new();
    if a.to != b.to {
        changes.push(ChangeEntry::new(
            "exit",
            "target_changed",
            id,
            serde_json::json!({
                "before": a.to,
                "after": b.to,
            }),
        ));
    }
    if a.is_conditional != b.is_conditional || a.guard_count != b.guard_count {
        changes.push(ChangeEntry::new(
            "exit",
            "condition_changed",
            id,
            serde_json::json!({
                "before": { "is_conditional": a.is_conditional, "guard_count": a.guard_count },
                "after": { "is_conditional": b.is_conditional, "guard_count": b.guard_count },
            }),
        ));
    }
    changes
}

fn compare_section(
    _category: &str,
    id: &str,
    a: &SectionSnapshot,
    b: &SectionSnapshot,
) -> Vec<ChangeEntry> {
    let mut changes = Vec::new();
    let mut a_targets = a.jump_targets.clone();
    let mut b_targets = b.jump_targets.clone();
    a_targets.sort();
    b_targets.sort();
    if a_targets != b_targets {
        changes.push(ChangeEntry::new(
            "section",
            "jumps_changed",
            id,
            serde_json::json!({
                "before": a.jump_targets,
                "after": b.jump_targets,
            }),
        ));
    }
    changes
}

fn compare_choice(
    _category: &str,
    id: &str,
    a: &ChoiceSnapshot,
    b: &ChoiceSnapshot,
) -> Vec<ChangeEntry> {
    let mut changes = Vec::new();
    if a.label != b.label {
        changes.push(ChangeEntry::new(
            "choice",
            "label_changed",
            id,
            serde_json::json!({
                "before": a.label,
                "after": b.label,
            }),
        ));
    }
    if a.sticky != b.sticky {
        changes.push(ChangeEntry::new(
            "choice",
            "sticky_changed",
            id,
            serde_json::json!({
                "before": a.sticky,
                "after": b.sticky,
            }),
        ));
    }
    if a.condition_count != b.condition_count {
        changes.push(ChangeEntry::new(
            "choice",
            "guard_changed",
            id,
            serde_json::json!({
                "before": { "condition_count": a.condition_count },
                "after": { "condition_count": b.condition_count },
            }),
        ));
    }
    if a.effect_count != b.effect_count {
        changes.push(ChangeEntry::new(
            "choice",
            "effect_changed",
            id,
            serde_json::json!({
                "before": { "effect_count": a.effect_count },
                "after": { "effect_count": b.effect_count },
            }),
        ));
    }
    let mut a_targets = a.jump_targets.clone();
    let mut b_targets = b.jump_targets.clone();
    a_targets.sort();
    b_targets.sort();
    if a_targets != b_targets {
        changes.push(ChangeEntry::new(
            "choice",
            "target_changed",
            id,
            serde_json::json!({
                "before": a.jump_targets,
                "after": b.jump_targets,
            }),
        ));
    }
    changes
}

fn compare_rule(
    _category: &str,
    id: &str,
    a: &RuleSnapshot,
    b: &RuleSnapshot,
) -> Vec<ChangeEntry> {
    let mut changes = Vec::new();
    if a.condition_count != b.condition_count {
        changes.push(ChangeEntry::new(
            "rule",
            "trigger_changed",
            id,
            serde_json::json!({
                "before": { "condition_count": a.condition_count },
                "after": { "condition_count": b.condition_count },
            }),
        ));
    }
    if a.effect_count != b.effect_count {
        changes.push(ChangeEntry::new(
            "rule",
            "effect_changed",
            id,
            serde_json::json!({
                "before": { "effect_count": a.effect_count },
                "after": { "effect_count": b.effect_count },
            }),
        ));
    }
    changes
}

fn compare_property(
    _category: &str,
    id: &str,
    a: &PropertySnapshot,
    b: &PropertySnapshot,
) -> Vec<ChangeEntry> {
    let mut changes = Vec::new();
    if a.read_count < b.read_count {
        changes.push(ChangeEntry::new(
            "property_dependency",
            "reader_added",
            id,
            serde_json::json!({
                "before": { "read_count": a.read_count },
                "after": { "read_count": b.read_count },
            }),
        ));
    } else if a.read_count > b.read_count {
        changes.push(ChangeEntry::new(
            "property_dependency",
            "reader_removed",
            id,
            serde_json::json!({
                "before": { "read_count": a.read_count },
                "after": { "read_count": b.read_count },
            }),
        ));
    }
    if a.write_count < b.write_count {
        changes.push(ChangeEntry::new(
            "property_dependency",
            "writer_added",
            id,
            serde_json::json!({
                "before": { "write_count": a.write_count },
                "after": { "write_count": b.write_count },
            }),
        ));
    } else if a.write_count > b.write_count {
        changes.push(ChangeEntry::new(
            "property_dependency",
            "writer_removed",
            id,
            serde_json::json!({
                "before": { "write_count": a.write_count },
                "after": { "write_count": b.write_count },
            }),
        ));
    }
    if a.orphaned != b.orphaned {
        changes.push(ChangeEntry::new(
            "property_dependency",
            "orphan_status_changed",
            id,
            serde_json::json!({
                "before": a.orphaned,
                "after": b.orphaned,
            }),
        ));
    }
    changes
}

/// Compare diagnostic key sets for reachability changes.
fn diff_diagnostic_keys(
    a: &BTreeSet<DiagnosticKey>,
    b: &BTreeSet<DiagnosticKey>,
    changes: &mut Vec<ChangeEntry>,
) {
    // In B but not A: newly appeared diagnostics.
    for key in b.difference(a) {
        let kind = match key.code.as_str() {
            "URD430" => "became_unreachable",
            "URD432" => "choice_became_impossible",
            _ => continue,
        };
        let element_type = match key.code.as_str() {
            "URD430" => "location",
            "URD432" => "section",
            _ => continue,
        };
        changes.push(ChangeEntry::new(
            "reachability",
            kind,
            &key.target_id,
            serde_json::json!({ "element_type": element_type }),
        ));
    }

    // In A but not B: resolved diagnostics.
    for key in a.difference(b) {
        let kind = match key.code.as_str() {
            "URD430" => "became_reachable",
            "URD432" => "choice_became_possible",
            _ => continue,
        };
        let element_type = match key.code.as_str() {
            "URD430" => "location",
            "URD432" => "section",
            _ => continue,
        };
        changes.push(ChangeEntry::new(
            "reachability",
            kind,
            &key.target_id,
            serde_json::json!({ "element_type": element_type }),
        ));
    }
}

// ── DiffReport JSON output ──

impl DiffReport {
    /// Serialise the report to structured JSON.
    pub fn to_json(&self) -> serde_json::Value {
        let changes: Vec<Json> = self
            .changes
            .iter()
            .map(|c| {
                serde_json::json!({
                    "category": c.category,
                    "kind": c.kind,
                    "id": c.id,
                    "detail": c.detail,
                })
            })
            .collect();

        let mut by_category: IndexMap<String, usize> = IndexMap::new();
        for c in &self.changes {
            *by_category.entry(c.category.clone()).or_insert(0) += 1;
        }
        let by_cat_json: serde_json::Map<String, Json> = by_category
            .iter()
            .map(|(k, v)| (k.clone(), Json::Number((*v as u64).into())))
            .collect();

        serde_json::json!({
            "changes": changes,
            "summary": {
                "total_changes": self.changes.len(),
                "by_category": Json::Object(by_cat_json),
            }
        })
    }

    /// Human-readable summary string.
    pub fn summary(&self) -> String {
        if self.changes.is_empty() {
            return "No changes detected.".to_string();
        }

        let mut by_category: IndexMap<String, usize> = IndexMap::new();
        for c in &self.changes {
            *by_category.entry(c.category.clone()).or_insert(0) += 1;
        }

        let parts: Vec<String> = by_category
            .iter()
            .map(|(cat, count)| format!("{} {}", count, cat))
            .collect();

        format!(
            "{} changes: {}",
            self.changes.len(),
            parts.join(", ")
        )
    }
}
