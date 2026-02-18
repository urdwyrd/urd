/// The symbol table: global registry of declared names.
///
/// LINK populates it during the collection sub-pass and queries it during
/// the resolution sub-pass. VALIDATE reads it for type information. EMIT
/// reads it for ID generation.
///
/// All maps are `IndexMap` to preserve insertion order for deterministic output.

use indexmap::IndexMap;

use crate::span::Span;

/// A duplicate declaration recorded for diagnostic purposes.
/// The canonical (first) declaration remains in the namespace map.
#[derive(Debug, Clone)]
pub struct Duplicate {
    pub namespace: &'static str,
    pub name: String,
    pub declared_in: Span,
}

/// The compiler's global symbol table.
///
/// Seven ordered maps — types, entities, sections, locations, actions, rules,
/// sequences — each preserving insertion order for deterministic output.
/// Duplicates are tracked in a flat list for diagnostics only.
#[derive(Debug, Default)]
pub struct SymbolTable {
    pub types: IndexMap<String, TypeSymbol>,
    pub entities: IndexMap<String, EntitySymbol>,
    pub sections: IndexMap<String, SectionSymbol>,
    pub locations: IndexMap<String, LocationSymbol>,
    pub actions: IndexMap<String, ActionSymbol>,
    pub rules: IndexMap<String, RuleSymbol>,
    pub sequences: IndexMap<String, SequenceSymbol>,
    pub duplicates: Vec<Duplicate>,
    /// Resolved `world.start` → location ID (set by LINK, consumed by VALIDATE).
    pub world_start: Option<String>,
    /// Resolved `world.entry` → sequence ID (set by LINK, consumed by VALIDATE).
    pub world_entry: Option<String>,
}

// ── Symbol types ──

/// Property type discriminator (7 types per the spec).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropertyType {
    Boolean,
    Integer,
    Number,
    String,
    Enum,
    Ref,
    List,
}

/// Visibility discriminator for properties.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Visibility {
    Visible,
    Hidden,
}

impl Default for Visibility {
    fn default() -> Self {
        Self::Visible
    }
}

/// A scalar value in the symbol table (property defaults, overrides).
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Integer(i64),
    Number(f64),
    Boolean(bool),
    List(Vec<Value>),
    EntityRef(String),
}

/// A type definition symbol.
#[derive(Debug, Clone)]
pub struct TypeSymbol {
    pub name: String,
    pub traits: Vec<String>,
    pub properties: IndexMap<String, PropertySymbol>,
    pub declared_in: Span,
}

/// A property within a type.
#[derive(Debug, Clone)]
pub struct PropertySymbol {
    pub name: String,
    pub property_type: PropertyType,
    pub default: Option<Value>,
    pub visibility: Visibility,
    pub values: Option<Vec<String>>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub ref_type: Option<String>,
    pub element_type: Option<PropertyType>,
    pub element_values: Option<Vec<String>>,
    pub element_ref_type: Option<String>,
    pub declared_in: Span,
}

/// An entity declaration symbol.
#[derive(Debug, Clone)]
pub struct EntitySymbol {
    pub id: String,
    pub type_name: String,
    pub type_symbol: Option<String>,
    pub property_overrides: IndexMap<String, Value>,
    pub declared_in: Span,
}

/// A section symbol (`== name`).
#[derive(Debug, Clone)]
pub struct SectionSymbol {
    pub local_name: String,
    pub compiled_id: String,
    pub file_stem: String,
    pub choices: Vec<ChoiceSymbol>,
    pub declared_in: Span,
}

/// A choice within a section.
#[derive(Debug, Clone)]
pub struct ChoiceSymbol {
    pub label: String,
    pub compiled_id: String,
    pub sticky: bool,
    pub declared_in: Span,
}

/// A location symbol (`# Heading`).
#[derive(Debug, Clone)]
pub struct LocationSymbol {
    pub id: String,
    pub display_name: String,
    pub exits: IndexMap<String, ExitSymbol>,
    pub contains: Vec<String>,
    pub declared_in: Span,
}

/// An exit within a location.
#[derive(Debug, Clone)]
pub struct ExitSymbol {
    pub direction: String,
    pub destination: String,
    pub resolved_destination: Option<String>,
    /// Index into the AST's content nodes for the Condition, if present.
    pub condition_node: Option<AstNodeRef>,
    /// Index into the AST's content nodes for the BlockedMessage, if present.
    pub blocked_message_node: Option<AstNodeRef>,
    pub declared_in: Span,
}

/// A lightweight reference to an AST node (file path + node index).
#[derive(Debug, Clone)]
pub struct AstNodeRef {
    pub file: crate::span::FilePath,
    pub node_index: usize,
}

/// An action symbol (either frontmatter-declared or choice-derived).
#[derive(Debug, Clone)]
pub struct ActionSymbol {
    pub id: String,
    pub target: Option<String>,
    pub target_type: Option<String>,
    pub declared_in: Span,
}

/// A rule symbol.
#[derive(Debug, Clone)]
pub struct RuleSymbol {
    pub id: String,
    pub actor: String,
    pub trigger: String,
    pub select: Option<SelectDef>,
    pub declared_in: Span,
}

/// A sequence symbol (`## Heading`).
#[derive(Debug, Clone)]
pub struct SequenceSymbol {
    pub id: String,
    pub phases: Vec<PhaseSymbol>,
    pub declared_in: Span,
}

/// A phase within a sequence (`### Heading`).
#[derive(Debug, Clone)]
pub struct PhaseSymbol {
    pub id: String,
    pub advance: String,
    pub action: Option<String>,
    pub actions: Option<Vec<String>>,
    pub rule: Option<String>,
    pub declared_in: Span,
}

/// The `selects...from...where` definition stored on a `RuleSymbol`.
#[derive(Debug, Clone)]
pub struct SelectDef {
    pub variable: String,
    pub from: Vec<String>,
    pub where_clauses: Vec<crate::ast::ConditionExpr>,
    pub span: Span,
}
