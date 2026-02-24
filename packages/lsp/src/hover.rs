/// Hover handler — provides Markdown tooltips for Urd constructs.

use lsp_server::Connection;
use lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind};

use crate::cursor::{self, Reference};
use crate::world_state::{self, WorldState};
use urd_compiler::definition_index::DefinitionKind;
use urd_compiler::facts::PropertyKey;

pub fn handle(connection: &Connection, state: &WorldState, req: lsp_server::Request) {
    let params: lsp_types::HoverParams =
        serde_json::from_value(req.params.clone()).unwrap();

    let result = build_hover(state, &params);

    let response = lsp_server::Response::new_ok(req.id, result);
    connection
        .sender
        .send(lsp_server::Message::Response(response))
        .ok();
}

fn build_hover(state: &WorldState, params: &lsp_types::HoverParams) -> Option<Hover> {
    let uri = &params.text_document_position_params.text_document.uri;
    let position = &params.text_document_position_params.position;

    let path = world_state::uri_to_path(uri);
    let source = std::fs::read_to_string(&path).ok()?;
    let line = source.lines().nth(position.line as usize)?;

    let reference = cursor::identify_reference(line, position.character as usize)?;

    let content = match reference {
        Reference::Entity(id) => hover_entity(state, &id)?,
        Reference::EntityProperty(entity_id, property) => {
            let type_name = resolve_entity_type(state, &entity_id)?;
            hover_property(state, &type_name, &property)?
        }
        Reference::TypeProperty(type_name, property) => {
            hover_property(state, &type_name, &property)?
        }
        Reference::SectionJump(name) | Reference::SectionLabel(name) => {
            hover_section(state, &name)?
        }
        Reference::LocationHeading(name) => hover_location(state, &name)?,
    };

    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: content,
        }),
        range: None,
    })
}

fn hover_entity(state: &WorldState, entity_id: &str) -> Option<String> {
    let world = state.world_json.as_ref()?;
    let entity = &world["entities"][entity_id];
    if entity.is_null() {
        return None;
    }

    let type_name = entity["type"].as_str().unwrap_or("unknown");
    let mut lines = vec![format!("**@{}**: {}", entity_id, type_name)];

    // Container (which location holds this entity)
    if let Some(locations) = world["locations"].as_object() {
        for (loc_id, loc) in locations {
            if let Some(contains) = loc["contains"].as_array() {
                if contains.iter().any(|e| e.as_str() == Some(entity_id)) {
                    lines.push(format!("Container: {}", loc_id));
                    break;
                }
            }
        }
    }

    // Properties with their values
    if let Some(props) = entity["properties"].as_object() {
        if !props.is_empty() {
            let prop_list: Vec<String> = props
                .iter()
                .map(|(k, v)| format!("{}: {}", k, v))
                .collect();
            lines.push(format!("Properties: {}", prop_list.join(", ")));
        }
    }

    Some(lines.join("\n\n"))
}

fn hover_property(state: &WorldState, type_name: &str, property: &str) -> Option<String> {
    let index = state.definition_index.as_ref()?;
    let key = format!("prop:{}.{}", type_name, property);
    let entry = index.get(&key)?;

    let mut lines = Vec::new();

    // Type and default from DefinitionIndex
    if let DefinitionKind::Property {
        property_type,
        default_repr,
        ..
    } = &entry.kind
    {
        let default_str = default_repr
            .as_deref()
            .map(|d| format!(" (default: {})", d))
            .unwrap_or_default();
        lines.push(format!(
            "**{}.{}**: {}{}",
            type_name, property, property_type, default_str
        ));
    }

    // Read/write counts from PropertyDependencyIndex
    if let Some(prop_index) = state.property_index() {
        let prop_key = PropertyKey {
            entity_type: type_name.to_string(),
            property: property.to_string(),
        };
        let read_count = prop_index.reads_of(&prop_key).len();
        let write_count = prop_index.writes_of(&prop_key).len();
        lines.push(format!(
            "Read by: {} site{} | Written by: {} site{}",
            read_count,
            if read_count == 1 { "" } else { "s" },
            write_count,
            if write_count == 1 { "" } else { "s" },
        ));

        // Orphan status
        let read_never_written = prop_index.read_but_never_written();
        let written_never_read = prop_index.written_but_never_read();
        if read_never_written.iter().any(|k| k.entity_type == type_name && k.property == property) {
            lines.push("Status: read but never written".to_string());
        } else if written_never_read.iter().any(|k| k.entity_type == type_name && k.property == property) {
            lines.push("Status: written but never read".to_string());
        } else if read_count > 0 && write_count > 0 {
            lines.push("Status: balanced".to_string());
        }
    }

    Some(lines.join("\n\n"))
}

fn hover_section(state: &WorldState, local_name: &str) -> Option<String> {
    let index = state.definition_index.as_ref()?;

    // Find the first section matching this local_name
    let (section_key, _entry) = index.iter().find(|(_, entry)| {
        matches!(&entry.kind, DefinitionKind::Section { local_name: ln, .. } if ln == local_name)
    })?;

    // Strip "section:" prefix to get compiled_id
    let compiled_id = section_key.strip_prefix("section:")?;

    let mut lines = vec![format!("**Section**: {}", compiled_id)];

    // Count incoming/outgoing jumps and choices from FactSet
    if let Some(fact_set) = state.fact_set() {
        let incoming = fact_set.jumps().iter().filter(|j| {
            matches!(&j.target, urd_compiler::facts::JumpTarget::Section(id) if id == compiled_id)
        }).count();

        let outgoing = fact_set.jumps().iter().filter(|j| {
            j.from_section == compiled_id
        }).count();

        let choices = fact_set.choices().iter().filter(|c| {
            c.section == compiled_id
        }).count();

        lines.push(format!(
            "Incoming jumps: {} | Outgoing jumps: {}",
            incoming, outgoing
        ));
        lines.push(format!("Choices: {}", choices));
    }

    Some(lines.join("\n\n"))
}

fn hover_location(state: &WorldState, display_name: &str) -> Option<String> {
    let index = state.definition_index.as_ref()?;

    // Find location by display_name
    let (location_key, _entry) = index.iter().find(|(_, entry)| {
        matches!(&entry.kind, DefinitionKind::Location { display_name: dn } if dn == display_name)
    })?;

    let slug = location_key.strip_prefix("location:")?;

    let mut lines = vec![format!("**Location**: {}", slug)];

    // Count exits and entities from world JSON
    if let Some(world) = &state.world_json {
        if let Some(location) = world["locations"].get(slug) {
            if let Some(exits) = location["exits"].as_array() {
                lines.push(format!("Exits: {}", exits.len()));
            }
            if let Some(contains) = location["contains"].as_array() {
                lines.push(format!("Entities: {}", contains.len()));
            }
        }
    }

    Some(lines.join("\n\n"))
}

fn resolve_entity_type(state: &WorldState, entity_id: &str) -> Option<String> {
    let world = state.world_json.as_ref()?;
    world["entities"][entity_id]["type"]
        .as_str()
        .map(|s| s.to_string())
}
