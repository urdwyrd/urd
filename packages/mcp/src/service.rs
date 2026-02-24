/// MCP service — tool definitions and dispatch via rmcp.

use std::sync::Arc;

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{ServerCapabilities, ServerInfo};
use schemars::JsonSchema;
use rmcp::{tool, tool_handler, tool_router, ServerHandler};
use serde::Deserialize;

use crate::queries;
use crate::world_data::WorldData;

// ── Parameter structs ──

#[derive(Debug, Deserialize, JsonSchema)]
pub struct EntityParams {
    #[schemars(description = "Entity ID (e.g., '@warden')")]
    pub entity_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PropertyParams {
    #[schemars(description = "Type name (e.g., 'Character')")]
    pub entity_type: String,
    #[schemars(description = "Property name (e.g., 'trust')")]
    pub property: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct LocationParams {
    #[schemars(description = "Starting location slug (e.g., 'gatehouse')")]
    pub from: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SectionParams {
    #[schemars(description = "Section compiled ID (e.g., 'gatehouse/greet')")]
    pub section: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DiagnosticParams {
    #[schemars(description = "Filter by severity: 'error', 'warning', or 'info'")]
    pub severity: Option<String>,
    #[schemars(description = "Filter by source file name")]
    pub file: Option<String>,
}

// ── Service struct ──

#[derive(Clone)]
pub struct UrdMcpService {
    data: Arc<WorldData>,
    tool_router: ToolRouter<Self>,
}

impl UrdMcpService {
    pub fn new(data: WorldData) -> Self {
        Self {
            data: Arc::new(data),
            tool_router: Self::tool_router(),
        }
    }
}

// ── Tool definitions ──

#[tool_router]
impl UrdMcpService {
    #[tool(
        name = "get_world_metadata",
        description = "Returns overview information about the compiled Urd world: name, version, start location, and counts of entities, locations, types, sections, exits, and rules."
    )]
    fn get_world_metadata(&self) -> String {
        queries::get_world_metadata(&self.data).to_string()
    }

    #[tool(
        name = "get_exit_graph",
        description = "Returns the complete location exit graph as nodes (locations) and edges (exits with direction, destination, and condition information). Use this to understand spatial navigation between locations."
    )]
    fn get_exit_graph(&self) -> String {
        queries::get_exit_graph(&self.data).to_string()
    }

    #[tool(
        name = "get_dialogue_graph",
        description = "Returns the dialogue structure graph: sections (dialogue nodes), jumps between sections, and choices within sections including their labels, conditions, and effects."
    )]
    fn get_dialogue_graph(&self) -> String {
        queries::get_dialogue_graph(&self.data).to_string()
    }

    #[tool(
        name = "get_entity_details",
        description = "Returns detailed information about a specific entity: its type, container location, and all properties with types, defaults, and constraints. Entity IDs start with '@'."
    )]
    fn get_entity_details(
        &self,
        Parameters(params): Parameters<EntityParams>,
    ) -> String {
        queries::get_entity_details(&self.data, &params.entity_id).to_string()
    }

    #[tool(
        name = "get_property_dependencies",
        description = "Returns where a specific property is read and written across the world. Shows all condition sites (reads) and effect sites (writes) with their expressions. Useful for understanding how a property influences game logic."
    )]
    fn get_property_dependencies(
        &self,
        Parameters(params): Parameters<PropertyParams>,
    ) -> String {
        queries::get_property_dependencies(&self.data, &params.entity_type, &params.property)
            .to_string()
    }

    #[tool(
        name = "get_reachable_locations",
        description = "Returns which locations are reachable from a starting location by following exits (ignoring conditions). Includes shortest paths. Useful for understanding world connectivity and finding isolated areas."
    )]
    fn get_reachable_locations(
        &self,
        Parameters(params): Parameters<LocationParams>,
    ) -> String {
        queries::get_reachable_locations(&self.data, &params.from).to_string()
    }

    #[tool(
        name = "get_choice_conditions",
        description = "Returns all choices in a dialogue section with their conditions (property reads) and effects (property writes). Useful for understanding what gates player options and what consequences they have."
    )]
    fn get_choice_conditions(
        &self,
        Parameters(params): Parameters<SectionParams>,
    ) -> String {
        queries::get_choice_conditions(&self.data, &params.section).to_string()
    }

    #[tool(
        name = "get_diagnostics",
        description = "Returns all compiler diagnostics (errors, warnings, info) for the compiled world. Optionally filter by severity or file."
    )]
    fn get_diagnostics(
        &self,
        Parameters(params): Parameters<DiagnosticParams>,
    ) -> String {
        queries::get_diagnostics(
            &self.data,
            params.severity.as_deref(),
            params.file.as_deref(),
        )
        .to_string()
    }
}

// ── ServerHandler ──

#[tool_handler]
impl ServerHandler for UrdMcpService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Read-only query interface for a compiled Urd world. \
                 Provides structural analysis tools for locations, entities, \
                 dialogue, properties, and diagnostics."
                    .to_string(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
