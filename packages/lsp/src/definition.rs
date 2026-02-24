/// Go-to-definition handler.
///
/// Resolves the identifier under the cursor and returns the declaration
/// location from the DefinitionIndex.

use lsp_server::Connection;

use crate::cursor::{self, Reference};
use crate::world_state::{self, WorldState};
use urd_compiler::definition_index::DefinitionKind;

pub fn handle(connection: &Connection, state: &WorldState, req: lsp_server::Request) {
    let params: lsp_types::GotoDefinitionParams =
        serde_json::from_value(req.params.clone()).unwrap();

    let result = find_definition(state, &params);

    let response = lsp_server::Response::new_ok(req.id, result);
    connection
        .sender
        .send(lsp_server::Message::Response(response))
        .ok();
}

fn find_definition(
    state: &WorldState,
    params: &lsp_types::GotoDefinitionParams,
) -> Option<lsp_types::GotoDefinitionResponse> {
    let index = state.definition_index.as_ref()?;
    let entry_dir = state.entry_dir()?;
    let uri = &params.text_document_position_params.text_document.uri;
    let position = &params.text_document_position_params.position;

    // Read the source line from disk
    let path = world_state::uri_to_path(uri);
    let source = std::fs::read_to_string(&path).ok()?;
    let line = source.lines().nth(position.line as usize)?;

    // Identify what's under the cursor
    let reference = cursor::identify_reference(line, position.character as usize)?;

    match reference {
        Reference::Entity(id) => {
            let key = format!("entity:@{}", id);
            let entry = index.get(&key)?;
            let location = world_state::span_to_location(&entry.span, &entry_dir);
            Some(lsp_types::GotoDefinitionResponse::Scalar(location))
        }
        Reference::EntityProperty(entity_id, property) => {
            // Resolve entity → type via world_json
            let type_name = resolve_entity_type(state, &entity_id)?;
            let key = format!("prop:{}.{}", type_name, property);
            let entry = index.get(&key)?;
            let location = world_state::span_to_location(&entry.span, &entry_dir);
            Some(lsp_types::GotoDefinitionResponse::Scalar(location))
        }
        Reference::TypeProperty(type_name, property) => {
            let key = format!("prop:{}.{}", type_name, property);
            let entry = index.get(&key)?;
            let location = world_state::span_to_location(&entry.span, &entry_dir);
            Some(lsp_types::GotoDefinitionResponse::Scalar(location))
        }
        Reference::SectionJump(name) => {
            // Find all sections with matching local_name
            let locations: Vec<lsp_types::Location> = index
                .iter()
                .filter_map(|(_, entry)| match &entry.kind {
                    DefinitionKind::Section { local_name, .. } if local_name == &name => {
                        Some(world_state::span_to_location(&entry.span, &entry_dir))
                    }
                    _ => None,
                })
                .collect();

            match locations.len() {
                0 => None,
                1 => Some(lsp_types::GotoDefinitionResponse::Scalar(
                    locations.into_iter().next().unwrap(),
                )),
                _ => Some(lsp_types::GotoDefinitionResponse::Array(locations)),
            }
        }
        Reference::SectionLabel(name) => {
            // Section label — jump to itself or find by local_name
            let locations: Vec<lsp_types::Location> = index
                .iter()
                .filter_map(|(_, entry)| match &entry.kind {
                    DefinitionKind::Section { local_name, .. } if local_name == &name => {
                        Some(world_state::span_to_location(&entry.span, &entry_dir))
                    }
                    _ => None,
                })
                .collect();

            match locations.len() {
                0 => None,
                1 => Some(lsp_types::GotoDefinitionResponse::Scalar(
                    locations.into_iter().next().unwrap(),
                )),
                _ => Some(lsp_types::GotoDefinitionResponse::Array(locations)),
            }
        }
        Reference::LocationHeading(name) => {
            // Find location by display_name
            let locations: Vec<lsp_types::Location> = index
                .iter()
                .filter_map(|(_, entry)| match &entry.kind {
                    DefinitionKind::Location { display_name } if display_name == &name => {
                        Some(world_state::span_to_location(&entry.span, &entry_dir))
                    }
                    _ => None,
                })
                .collect();

            match locations.len() {
                0 => None,
                1 => Some(lsp_types::GotoDefinitionResponse::Scalar(
                    locations.into_iter().next().unwrap(),
                )),
                _ => Some(lsp_types::GotoDefinitionResponse::Array(locations)),
            }
        }
    }
}

/// Resolve an entity ID to its type name via the compiled world JSON.
fn resolve_entity_type(state: &WorldState, entity_id: &str) -> Option<String> {
    let world = state.world_json.as_ref()?;
    world["entities"][entity_id]["type"]
        .as_str()
        .map(|s| s.to_string())
}
