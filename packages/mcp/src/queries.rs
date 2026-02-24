/// Pure query functions: (WorldData, params) → serde_json::Value.
///
/// Each function is self-contained, read-only, and includes
/// `"schema_version": "1"` in every response.

use std::collections::{HashMap, HashSet, VecDeque};

use serde_json::{json, Value};

use urd_compiler::facts::{CompareOp, FactSite, JumpTarget, WriteOp};

use crate::world_data::WorldData;

// ── Helpers ──

fn compare_op_symbol(op: &CompareOp) -> &'static str {
    match op {
        CompareOp::Eq => "==",
        CompareOp::Ne => "!=",
        CompareOp::Lt => "<",
        CompareOp::Gt => ">",
        CompareOp::Le => "<=",
        CompareOp::Ge => ">=",
    }
}

fn write_op_symbol(op: &WriteOp) -> &'static str {
    match op {
        WriteOp::Set => "=",
        WriteOp::Add => "+=",
        WriteOp::Sub => "-=",
    }
}

fn format_site(site: &FactSite) -> String {
    match site {
        FactSite::Choice(id) => format!("choice:{}", id),
        FactSite::Exit(id) => format!("exit:{}", id),
        FactSite::Rule(id) => format!("rule:{}", id),
        _ => format!("unknown"),
    }
}

fn object_keys_sorted(value: &Value, key: &str) -> Vec<String> {
    value[key]
        .as_object()
        .map(|obj| {
            let mut keys: Vec<String> = obj.keys().cloned().collect();
            keys.sort();
            keys
        })
        .unwrap_or_default()
}

// ── Tool 1: get_world_metadata ──

pub fn get_world_metadata(data: &WorldData) -> Value {
    let (world_name, start_location) = match &data.world_json {
        Some(w) => (
            w["world"]["name"].as_str().unwrap_or("unknown").to_string(),
            w["world"]["start"]
                .as_str()
                .unwrap_or("unknown")
                .to_string(),
        ),
        None => ("unknown".to_string(), "unknown".to_string()),
    };

    let entity_count = data
        .world_json
        .as_ref()
        .and_then(|w| w["entities"].as_object())
        .map(|o| o.len())
        .unwrap_or(0);

    let location_count = data
        .world_json
        .as_ref()
        .and_then(|w| w["locations"].as_object())
        .map(|o| o.len())
        .unwrap_or(0);

    let type_count = data
        .world_json
        .as_ref()
        .and_then(|w| w["types"].as_object())
        .map(|o| o.len())
        .unwrap_or(0);

    let section_count = data
        .world_json
        .as_ref()
        .and_then(|w| w["dialogue"].as_object())
        .map(|o| o.len())
        .unwrap_or(0);

    let exit_count = data
        .fact_set
        .as_ref()
        .map(|fs| fs.exits().len())
        .unwrap_or(0);

    let rule_count = data
        .fact_set
        .as_ref()
        .map(|fs| fs.rules().len())
        .unwrap_or(0);

    json!({
        "schema_version": "1",
        "world_name": world_name,
        "start_location": start_location,
        "entity_count": entity_count,
        "location_count": location_count,
        "type_count": type_count,
        "section_count": section_count,
        "exit_count": exit_count,
        "rule_count": rule_count,
        "has_errors": data.has_errors,
        "diagnostic_count": data.diagnostics.len()
    })
}

// ── Tool 2: get_exit_graph ──

pub fn get_exit_graph(data: &WorldData) -> Value {
    let nodes = data
        .world_json
        .as_ref()
        .map(|w| object_keys_sorted(w, "locations"))
        .unwrap_or_default();

    let edges: Vec<Value> = data
        .fact_set
        .as_ref()
        .map(|fs| {
            fs.exits()
                .iter()
                .map(|e| {
                    json!({
                        "from": e.from_location,
                        "to": e.to_location,
                        "exit_name": e.exit_name,
                        "is_conditional": e.is_conditional,
                        "guard_count": e.guard_reads.len()
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    json!({
        "schema_version": "1",
        "nodes": nodes,
        "edges": edges
    })
}

// ── Tool 3: get_dialogue_graph ──

pub fn get_dialogue_graph(data: &WorldData) -> Value {
    let sections = data
        .world_json
        .as_ref()
        .map(|w| object_keys_sorted(w, "dialogue"))
        .unwrap_or_default();

    let jumps: Vec<Value> = data
        .fact_set
        .as_ref()
        .map(|fs| {
            fs.jumps()
                .iter()
                .map(|j| {
                    let (to_section, jump_type) = match &j.target {
                        JumpTarget::Section(id) => (id.as_str(), "section"),
                        JumpTarget::Exit(id) => (id.as_str(), "exit"),
                        JumpTarget::End => ("end", "end"),
                    };
                    json!({
                        "from_section": j.from_section,
                        "to_section": to_section,
                        "type": jump_type
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let choices: Vec<Value> = data
        .fact_set
        .as_ref()
        .map(|fs| {
            fs.choices()
                .iter()
                .map(|c| {
                    json!({
                        "section": c.section,
                        "choice_id": c.choice_id,
                        "label": c.label,
                        "sticky": c.sticky,
                        "condition_count": c.condition_reads.len(),
                        "effect_count": c.effect_writes.len()
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    json!({
        "schema_version": "1",
        "sections": sections,
        "jumps": jumps,
        "choices": choices
    })
}

// ── Tool 4: get_entity_details ──

pub fn get_entity_details(data: &WorldData, entity_id: &str) -> Value {
    let world = match &data.world_json {
        Some(w) => w,
        None => {
            return json!({
                "schema_version": "1",
                "error": "No compiled world available",
                "entity_id": entity_id
            })
        }
    };

    // Strip leading @ if present
    let clean_id = entity_id.strip_prefix('@').unwrap_or(entity_id);

    let entity = &world["entities"][clean_id];
    if entity.is_null() {
        return json!({
            "schema_version": "1",
            "error": "Entity not found",
            "entity_id": entity_id
        });
    }

    let type_name = entity["type"].as_str().unwrap_or("unknown");

    // Find container location
    let container = world["locations"]
        .as_object()
        .and_then(|locations| {
            locations.iter().find_map(|(loc_id, loc)| {
                loc["contains"].as_array().and_then(|contains| {
                    if contains.iter().any(|e| e.as_str() == Some(clean_id)) {
                        Some(loc_id.clone())
                    } else {
                        None
                    }
                })
            })
        })
        .unwrap_or_default();

    // Build property list from type definitions + entity overrides
    let type_props = &world["types"][type_name]["properties"];
    let entity_props = &entity["properties"];

    let mut properties: Vec<Value> = Vec::new();
    if let Some(props) = type_props.as_object() {
        for (name, prop_def) in props {
            let mut prop = json!({
                "name": name,
                "type": prop_def["type"].as_str().unwrap_or("unknown"),
            });

            // Default value (entity override or type default)
            if let Some(override_val) = entity_props.get(name) {
                prop["default"] = override_val.clone();
            } else if !prop_def["default"].is_null() {
                prop["default"] = prop_def["default"].clone();
            }

            // Enum values
            if let Some(values) = prop_def["values"].as_array() {
                prop["values"] = json!(values);
            }

            // Visibility
            if let Some(vis) = prop_def["visibility"].as_str() {
                if vis != "visible" {
                    prop["visibility"] = json!(vis);
                }
            }

            // Min/max
            if !prop_def["min"].is_null() {
                prop["min"] = prop_def["min"].clone();
            }
            if !prop_def["max"].is_null() {
                prop["max"] = prop_def["max"].clone();
            }

            properties.push(prop);
        }
    }

    json!({
        "schema_version": "1",
        "entity_id": format!("@{}", clean_id),
        "type": type_name,
        "container": container,
        "properties": properties
    })
}

// ── Tool 5: get_property_dependencies ──

pub fn get_property_dependencies(data: &WorldData, entity_type: &str, property: &str) -> Value {
    let fact_set = match &data.fact_set {
        Some(fs) => fs,
        None => {
            return json!({
                "schema_version": "1",
                "error": "No FactSet available",
                "property_key": format!("{}.{}", entity_type, property)
            })
        }
    };

    let prop_index = match &data.property_index {
        Some(pi) => pi,
        None => {
            return json!({
                "schema_version": "1",
                "error": "No PropertyDependencyIndex available",
                "property_key": format!("{}.{}", entity_type, property)
            })
        }
    };

    let key = urd_compiler::facts::PropertyKey {
        entity_type: entity_type.to_string(),
        property: property.to_string(),
    };

    let read_indices = prop_index.reads_of(&key);
    let write_indices = prop_index.writes_of(&key);

    if read_indices.is_empty() && write_indices.is_empty() {
        return json!({
            "schema_version": "1",
            "error": "Property not found",
            "property_key": format!("{}.{}", entity_type, property)
        });
    }

    let reads: Vec<Value> = read_indices
        .iter()
        .filter_map(|&idx| fact_set.reads().get(idx))
        .map(|r| {
            json!({
                "site": format_site(&r.site),
                "comparison": format!("{} {} {}", r.property, compare_op_symbol(&r.operator), r.value_literal)
            })
        })
        .collect();

    let writes: Vec<Value> = write_indices
        .iter()
        .filter_map(|&idx| fact_set.writes().get(idx))
        .map(|w| {
            json!({
                "site": format_site(&w.site),
                "operation": format!("{} {} {}", w.property, write_op_symbol(&w.operator), w.value_expr)
            })
        })
        .collect();

    let read_count = reads.len();
    let write_count = writes.len();

    json!({
        "schema_version": "1",
        "property_key": format!("{}.{}", entity_type, property),
        "read_count": read_count,
        "write_count": write_count,
        "read_only": read_count > 0 && write_count == 0,
        "write_only": write_count > 0 && read_count == 0,
        "reads": reads,
        "writes": writes
    })
}

// ── Tool 6: get_reachable_locations ──

pub fn get_reachable_locations(data: &WorldData, from: &str) -> Value {
    let world = match &data.world_json {
        Some(w) => w,
        None => {
            return json!({
                "schema_version": "1",
                "error": "No compiled world available",
                "location": from
            })
        }
    };

    let all_locations = object_keys_sorted(world, "locations");
    if !all_locations.contains(&from.to_string()) {
        return json!({
            "schema_version": "1",
            "error": "Location not found",
            "location": from
        });
    }

    let fact_set = match &data.fact_set {
        Some(fs) => fs,
        None => {
            return json!({
                "schema_version": "1",
                "from": from,
                "reachable": [from],
                "unreachable": all_locations.iter().filter(|l| l.as_str() != from).collect::<Vec<_>>(),
                "path_count": 1,
                "paths": {}
            })
        }
    };

    // Build adjacency map
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
    for exit in fact_set.exits() {
        adj.entry(exit.from_location.as_str())
            .or_default()
            .push(exit.to_location.as_str());
    }
    // Sort for deterministic BFS (alphabetical tie-breaking)
    for list in adj.values_mut() {
        list.sort();
        list.dedup();
    }

    // BFS
    let mut visited: HashSet<&str> = HashSet::new();
    let mut parent: HashMap<&str, &str> = HashMap::new();
    let mut queue: VecDeque<&str> = VecDeque::new();

    visited.insert(from);
    queue.push_back(from);

    while let Some(current) = queue.pop_front() {
        if let Some(neighbours) = adj.get(current) {
            for &next in neighbours {
                if visited.insert(next) {
                    parent.insert(next, current);
                    queue.push_back(next);
                }
            }
        }
    }

    let mut reachable: Vec<String> = visited.iter().map(|s| s.to_string()).collect();
    reachable.sort();

    let all_set: HashSet<&str> = all_locations.iter().map(|s| s.as_str()).collect();
    let mut unreachable: Vec<String> = all_set
        .difference(&visited)
        .map(|s| s.to_string())
        .collect();
    unreachable.sort();

    // Reconstruct shortest paths
    let mut paths: serde_json::Map<String, Value> = serde_json::Map::new();
    for loc in &reachable {
        if loc == from {
            continue;
        }
        let mut path = vec![loc.clone()];
        let mut current = loc.as_str();
        while let Some(&p) = parent.get(current) {
            path.push(p.to_string());
            current = p;
        }
        path.reverse();
        paths.insert(loc.clone(), json!(path));
    }

    json!({
        "schema_version": "1",
        "from": from,
        "reachable": reachable,
        "unreachable": unreachable,
        "path_count": reachable.len(),
        "paths": Value::Object(paths)
    })
}

// ── Tool 7: get_choice_conditions ──

pub fn get_choice_conditions(data: &WorldData, section: &str) -> Value {
    let fact_set = match &data.fact_set {
        Some(fs) => fs,
        None => {
            return json!({
                "schema_version": "1",
                "error": "No FactSet available",
                "section": section
            })
        }
    };

    // Check if section exists in world JSON
    let section_exists = data
        .world_json
        .as_ref()
        .and_then(|w| w["dialogue"].as_object())
        .map(|d| d.contains_key(section))
        .unwrap_or(false);

    let matching_choices: Vec<_> = fact_set
        .choices()
        .iter()
        .filter(|c| c.section == section)
        .collect();

    if matching_choices.is_empty() && !section_exists {
        return json!({
            "schema_version": "1",
            "error": "Section not found",
            "section": section
        });
    }

    let choices: Vec<Value> = matching_choices
        .iter()
        .map(|c| {
            let conditions: Vec<Value> = c
                .condition_reads
                .iter()
                .filter_map(|&idx| fact_set.reads().get(idx))
                .map(|r| {
                    json!({
                        "property": format!("{}.{}", r.entity_type, r.property),
                        "comparison": format!("{} {}", compare_op_symbol(&r.operator), r.value_literal)
                    })
                })
                .collect();

            let effects: Vec<Value> = c
                .effect_writes
                .iter()
                .filter_map(|&idx| fact_set.writes().get(idx))
                .map(|w| {
                    json!({
                        "property": format!("{}.{}", w.entity_type, w.property),
                        "operation": format!("{} {}", write_op_symbol(&w.operator), w.value_expr)
                    })
                })
                .collect();

            json!({
                "choice_id": c.choice_id,
                "label": c.label,
                "sticky": c.sticky,
                "conditions": conditions,
                "effects": effects
            })
        })
        .collect();

    json!({
        "schema_version": "1",
        "section": section,
        "choices": choices
    })
}

// ── Tool 8: get_diagnostics ──

pub fn get_diagnostics(data: &WorldData, severity: Option<&str>, file: Option<&str>) -> Value {
    let filtered: Vec<&crate::world_data::DiagnosticEntry> = data
        .diagnostics
        .iter()
        .filter(|d| {
            if let Some(sev) = severity {
                if d.severity != sev {
                    return false;
                }
            }
            if let Some(f) = file {
                if d.file != f {
                    return false;
                }
            }
            true
        })
        .collect();

    let errors = filtered.iter().filter(|d| d.severity == "error").count();
    let warnings = filtered.iter().filter(|d| d.severity == "warning").count();
    let info = filtered.iter().filter(|d| d.severity == "info").count();

    let diagnostics: Vec<Value> = filtered
        .iter()
        .map(|d| {
            json!({
                "severity": d.severity,
                "code": d.code,
                "message": d.message,
                "file": d.file,
                "start_line": d.start_line,
                "start_col": d.start_col,
                "end_line": d.end_line,
                "end_col": d.end_col
            })
        })
        .collect();

    json!({
        "schema_version": "1",
        "total": filtered.len(),
        "errors": errors,
        "warnings": warnings,
        "info": info,
        "diagnostics": diagnostics
    })
}
