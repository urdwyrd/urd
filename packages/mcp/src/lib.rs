/// Urd MCP Server — read-only semantic query interface for compiled worlds.
///
/// Exposes eight tools via the Model Context Protocol, backed by FactSet,
/// PropertyDependencyIndex, and compiled world JSON. Read-only, no mutation.

pub mod queries;
pub mod service;
pub mod world_data;
