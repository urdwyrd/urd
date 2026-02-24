/// Definition index: maps namespace-prefixed keys to declaration spans.
///
/// Built from the SymbolTable after LINK. Provides go-to-definition and
/// hover data for the LSP server without exposing AST or SymbolTable internals.
///
/// Key format: `namespace:identifier` — e.g. `entity:@warden`, `prop:Guard.trust`,
/// `section:gatehouse/greet`, `location:village-square`.

use indexmap::IndexMap;

use crate::span::Span;
use crate::symbol_table::SymbolTable;

/// The kind of definition an entry represents.
#[derive(Debug, Clone)]
pub enum DefinitionKind {
    Type,
    Entity { type_name: String },
    Property {
        type_name: String,
        property_type: String,
        default_repr: Option<String>,
    },
    Section {
        local_name: String,
        file_stem: String,
    },
    Location {
        display_name: String,
    },
    Exit {
        from_location: String,
        destination: String,
    },
    Choice {
        section_id: String,
        label: String,
    },
    Rule,
}

/// A single definition entry: a declaration span plus its kind metadata.
#[derive(Debug, Clone)]
pub struct DefinitionEntry {
    pub span: Span,
    pub kind: DefinitionKind,
}

/// Index mapping namespace-prefixed keys to their declaration sites.
///
/// Built once after LINK and stored in `CompilationResult`. Consumers
/// (LSP, playground) use `get()` for direct lookup or `iter()` for scanning.
#[derive(Debug, Clone)]
pub struct DefinitionIndex {
    entries: IndexMap<String, DefinitionEntry>,
}

impl DefinitionIndex {
    /// Build the index from a resolved SymbolTable.
    ///
    /// Iterates all seven SymbolTable maps plus nested structures
    /// (properties, choices, exits) to produce namespace-prefixed keys.
    pub fn build(symbol_table: &SymbolTable) -> Self {
        let mut entries = IndexMap::new();

        // Types: key = "type:Name"
        for (name, sym) in &symbol_table.types {
            entries.insert(
                format!("type:{}", name),
                DefinitionEntry {
                    span: sym.declared_in.clone(),
                    kind: DefinitionKind::Type,
                },
            );

            // Properties: key = "prop:Type.name"
            for (prop_name, prop_sym) in &sym.properties {
                let default_repr = prop_sym.default.as_ref().map(format_value);
                entries.insert(
                    format!("prop:{}.{}", name, prop_name),
                    DefinitionEntry {
                        span: prop_sym.declared_in.clone(),
                        kind: DefinitionKind::Property {
                            type_name: name.clone(),
                            property_type: prop_sym.raw_type_string.clone(),
                            default_repr,
                        },
                    },
                );
            }
        }

        // Entities: key = "entity:@id"
        for (id, sym) in &symbol_table.entities {
            entries.insert(
                format!("entity:@{}", id),
                DefinitionEntry {
                    span: sym.declared_in.clone(),
                    kind: DefinitionKind::Entity {
                        type_name: sym.type_name.clone(),
                    },
                },
            );
        }

        // Sections: key = "section:compiled_id"
        for (compiled_id, sym) in &symbol_table.sections {
            entries.insert(
                format!("section:{}", compiled_id),
                DefinitionEntry {
                    span: sym.declared_in.clone(),
                    kind: DefinitionKind::Section {
                        local_name: sym.local_name.clone(),
                        file_stem: sym.file_stem.clone(),
                    },
                },
            );

            // Choices: key = "choice:compiled_id"
            for choice_sym in &sym.choices {
                entries.insert(
                    format!("choice:{}", choice_sym.compiled_id),
                    DefinitionEntry {
                        span: choice_sym.declared_in.clone(),
                        kind: DefinitionKind::Choice {
                            section_id: compiled_id.clone(),
                            label: choice_sym.label.clone(),
                        },
                    },
                );
            }
        }

        // Locations: key = "location:slug"
        for (slug, sym) in &symbol_table.locations {
            entries.insert(
                format!("location:{}", slug),
                DefinitionEntry {
                    span: sym.declared_in.clone(),
                    kind: DefinitionKind::Location {
                        display_name: sym.display_name.clone(),
                    },
                },
            );

            // Exits: key = "exit:location/direction"
            for (direction, exit_sym) in &sym.exits {
                let destination = exit_sym
                    .resolved_destination
                    .as_deref()
                    .unwrap_or(&exit_sym.destination);
                entries.insert(
                    format!("exit:{}/{}", slug, direction),
                    DefinitionEntry {
                        span: exit_sym.declared_in.clone(),
                        kind: DefinitionKind::Exit {
                            from_location: slug.clone(),
                            destination: destination.to_string(),
                        },
                    },
                );
            }
        }

        // Rules: key = "rule:name"
        for (name, sym) in &symbol_table.rules {
            entries.insert(
                format!("rule:{}", name),
                DefinitionEntry {
                    span: sym.declared_in.clone(),
                    kind: DefinitionKind::Rule,
                },
            );
        }

        Self { entries }
    }

    /// Look up a definition by its namespace-prefixed key.
    pub fn get(&self, key: &str) -> Option<&DefinitionEntry> {
        self.entries.get(key)
    }

    /// All entries in insertion order.
    pub fn entries(&self) -> &IndexMap<String, DefinitionEntry> {
        &self.entries
    }

    /// Iterate over all (key, entry) pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &DefinitionEntry)> {
        self.entries.iter()
    }

    /// Number of definitions in the index.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the index is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Serialise the index to JSON.
    pub fn to_json(&self) -> serde_json::Value {
        let entries: Vec<serde_json::Value> = self
            .entries
            .iter()
            .map(|(key, entry)| {
                let kind_json = match &entry.kind {
                    DefinitionKind::Type => serde_json::json!({ "kind": "type" }),
                    DefinitionKind::Entity { type_name } => {
                        serde_json::json!({ "kind": "entity", "type_name": type_name })
                    }
                    DefinitionKind::Property {
                        type_name,
                        property_type,
                        default_repr,
                    } => serde_json::json!({
                        "kind": "property",
                        "type_name": type_name,
                        "property_type": property_type,
                        "default": default_repr,
                    }),
                    DefinitionKind::Section {
                        local_name,
                        file_stem,
                    } => serde_json::json!({
                        "kind": "section",
                        "local_name": local_name,
                        "file_stem": file_stem,
                    }),
                    DefinitionKind::Location { display_name } => {
                        serde_json::json!({ "kind": "location", "display_name": display_name })
                    }
                    DefinitionKind::Exit {
                        from_location,
                        destination,
                    } => serde_json::json!({
                        "kind": "exit",
                        "from_location": from_location,
                        "destination": destination,
                    }),
                    DefinitionKind::Choice { section_id, label } => serde_json::json!({
                        "kind": "choice",
                        "section_id": section_id,
                        "label": label,
                    }),
                    DefinitionKind::Rule => serde_json::json!({ "kind": "rule" }),
                };

                serde_json::json!({
                    "key": key,
                    "span": {
                        "file": entry.span.file,
                        "start_line": entry.span.start_line,
                        "start_col": entry.span.start_col,
                        "end_line": entry.span.end_line,
                        "end_col": entry.span.end_col,
                    },
                    "definition": kind_json,
                })
            })
            .collect();

        serde_json::json!({
            "definitions": entries,
            "count": self.entries.len(),
        })
    }
}

/// Format a Value for display in definition metadata.
fn format_value(v: &crate::symbol_table::Value) -> String {
    match v {
        crate::symbol_table::Value::String(s) => format!("\"{}\"", s),
        crate::symbol_table::Value::Integer(n) => n.to_string(),
        crate::symbol_table::Value::Number(n) => n.to_string(),
        crate::symbol_table::Value::Boolean(b) => b.to_string(),
        crate::symbol_table::Value::EntityRef(r) => format!("@{}", r),
        crate::symbol_table::Value::List(items) => {
            let inner: Vec<String> = items.iter().map(format_value).collect();
            format!("[{}]", inner.join(", "))
        }
    }
}
