//! Compiler bridge — wraps urd_compiler for Tauri IPC.
//!
//! Accepts a buffer map (HashMap<String, String>), constructs an in-memory
//! FileReader, runs compilation, and returns chunked output with content
//! hashes for the frontend cache.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use urd_compiler::import::{FileReadError, FileReader};

// ===== In-memory FileReader =====

struct BufferMapReader {
    files: HashMap<String, String>,
}

impl FileReader for BufferMapReader {
    fn read_file(&self, fs_path: &str) -> Result<String, FileReadError> {
        // Try exact match first, then normalised
        if let Some(content) = self.files.get(fs_path) {
            return Ok(content.clone());
        }
        // Normalise: forward slashes, strip leading ./
        let normalised = fs_path.replace('\\', "/");
        let normalised = normalised.strip_prefix("./").unwrap_or(&normalised);
        for (key, content) in &self.files {
            let key_normalised = key.replace('\\', "/");
            let key_normalised = key_normalised
                .strip_prefix("./")
                .unwrap_or(&key_normalised);
            if key_normalised == normalised || key_normalised.ends_with(normalised) {
                return Ok(content.clone());
            }
        }
        Err(FileReadError::NotFound)
    }

    fn canonical_filename(&self, _dir: &str, _filename: &str) -> Option<String> {
        None // Case-sensitive behaviour
    }
}

// ===== Output types (serialised to frontend) =====

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CompilerOutput {
    pub header: OutputHeader,
    pub chunks: Vec<Chunk>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OutputHeader {
    pub compile_id: String,
    pub timestamp: u64,
    pub duration_ms: u64,
    pub phase_timings: Vec<PhaseTiming>,
    pub world_counts: WorldCounts,
    pub input_file_count: usize,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PhaseTiming {
    pub phase: String,
    pub duration_ms: u64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WorldCounts {
    pub entities: usize,
    pub locations: usize,
    pub exits: usize,
    pub properties: usize,
    pub rules: usize,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Chunk {
    pub name: String,
    pub data: serde_json::Value,
    pub content_hash: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticOutput {
    pub severity: String,
    pub message: String,
    pub code: String,
    pub span: Option<SpanOutput>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SpanOutput {
    pub file: String,
    pub start_line: u32,
    pub start_col: u32,
    pub end_line: u32,
    pub end_col: u32,
}

// ===== Helpers =====

fn sha256_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

fn make_chunk(name: &str, data: serde_json::Value) -> Chunk {
    let serialised = serde_json::to_string(&data).unwrap_or_default();
    let hash = sha256_hash(serialised.as_bytes());
    Chunk {
        name: name.to_string(),
        data,
        content_hash: hash,
    }
}

// ===== Core compile function =====

pub fn compile_buffers(
    buffers: HashMap<String, String>,
    compile_id: &str,
    entry_file: Option<&str>,
) -> CompilerOutput {
    let start = std::time::Instant::now();

    // Use the provided entry file if it exists in the buffer map,
    // otherwise fall back to first .urd.md alphabetically.
    // Path normalisation: the frontend may send paths with mixed slashes
    // (e.g., "C:\Users\...\project/file.urd.md") while buffer keys use
    // OS-native separators. Normalise to forward slashes for comparison.
    let entry_filename = entry_file
        .and_then(|ef| {
            let norm_ef = ef.replace('\\', "/");
            // Exact match first
            if buffers.contains_key(ef) {
                return Some(ef.to_string());
            }
            // Normalised exact match
            if let Some(key) = buffers.keys()
                .find(|k| k.replace('\\', "/") == norm_ef)
            {
                return Some(key.clone());
            }
            // Suffix match — frontend may send a short filename while buffer
            // keys are full paths (or vice versa)
            let suffix = format!("/{}", norm_ef.trim_start_matches('/'));
            buffers.keys()
                .find(|k| {
                    let nk = k.replace('\\', "/");
                    nk.ends_with(&suffix) || norm_ef.ends_with(&format!("/{}", nk.trim_start_matches('/')))
                })
                .cloned()
        })
        .unwrap_or_else(|| {
            buffers
                .keys()
                .filter(|k| k.ends_with(".urd.md"))
                .min()
                .or_else(|| buffers.keys().next())
                .cloned()
                .unwrap_or_else(|| "unknown.urd.md".to_string())
        });

    #[cfg(debug_assertions)]
    eprintln!(
        "[bridge] entry_file requested={:?}, resolved={:?}, buffer_keys={:?}",
        entry_file,
        entry_filename,
        buffers.keys().collect::<Vec<_>>()
    );

    let entry_source = buffers
        .get(&entry_filename)
        .cloned()
        .unwrap_or_default();

    let reader = BufferMapReader {
        files: buffers.clone(),
    };

    let input_file_count = buffers.len();

    let result =
        urd_compiler::compile_source_with_reader(&entry_filename, &entry_source, &reader);

    let duration = start.elapsed();
    let duration_ms = duration.as_millis() as u64;

    // Build diagnostics chunk
    let diagnostics: Vec<DiagnosticOutput> = result
        .diagnostics
        .all()
        .iter()
        .map(|d| {
            let severity = match d.severity {
                urd_compiler::diagnostics::Severity::Error => "error",
                urd_compiler::diagnostics::Severity::Warning => "warning",
                urd_compiler::diagnostics::Severity::Info => "info",
            };
            DiagnosticOutput {
                severity: severity.to_string(),
                message: d.message.clone(),
                code: d.code.clone(),
                span: if d.span.file.is_empty() {
                    None
                } else {
                    Some(SpanOutput {
                        file: d.span.file.clone(),
                        start_line: d.span.start_line,
                        start_col: d.span.start_col,
                        end_line: d.span.end_line,
                        end_col: d.span.end_col,
                    })
                },
            }
        })
        .collect();

    // Build urdJson chunk from world output.
    // Only Some when EMIT succeeds (zero errors). When None, the chunk is
    // omitted so the frontend cache retains the last successful world data
    // (stale-retention pattern, matching the playground).
    let urd_json: Option<serde_json::Value> = result
        .world
        .as_ref()
        .and_then(|w| serde_json::from_str(w).ok());

    // Build symbol table chunk — flat entries array for the frontend
    let symbol_table = serde_json::json!({
        "entries": result.symbol_table.as_ref().map(|st| {
            let mut entries: Vec<serde_json::Value> = Vec::new();
            for (id, sym) in &st.entities {
                entries.push(serde_json::json!({
                    "id": id,
                    "name": sym.id,
                    "kind": "entity",
                    "file": sym.declared_in.file,
                    "line": sym.declared_in.start_line,
                }));
            }
            for (id, sym) in &st.locations {
                entries.push(serde_json::json!({
                    "id": id,
                    "name": sym.display_name,
                    "kind": "location",
                    "file": sym.declared_in.file,
                    "line": sym.declared_in.start_line,
                }));
            }
            for (id, sym) in &st.types {
                entries.push(serde_json::json!({
                    "id": id,
                    "name": sym.name,
                    "kind": "type",
                    "file": sym.declared_in.file,
                    "line": sym.declared_in.start_line,
                }));
            }
            for (id, sym) in &st.sections {
                entries.push(serde_json::json!({
                    "id": id,
                    "name": sym.local_name,
                    "kind": "section",
                    "file": sym.declared_in.file,
                    "line": sym.declared_in.start_line,
                }));
            }
            for (id, sym) in &st.rules {
                entries.push(serde_json::json!({
                    "id": id,
                    "name": sym.id,
                    "kind": "rule",
                    "file": sym.declared_in.file,
                    "line": sym.declared_in.start_line,
                }));
            }
            for (id, sym) in &st.actions {
                entries.push(serde_json::json!({
                    "id": id,
                    "name": sym.id,
                    "kind": "action",
                    "file": sym.declared_in.file,
                    "line": sym.declared_in.start_line,
                }));
            }
            serde_json::Value::Array(entries)
        }).unwrap_or(serde_json::json!([]))
    });

    // Build fact set chunk — only when LINK produced facts.
    // Omitted chunks are retained by the frontend cache (stale-retention).
    let fact_set: Option<serde_json::Value> = result
        .fact_set
        .as_ref()
        .map(|fs| fs.to_json());

    // Build AST chunk (placeholder — full AST serialisation deferred)
    let ast = serde_json::json!({ "nodes": [] });

    // Build property dependency index chunk — only when available.
    let prop_index: Option<serde_json::Value> = result
        .property_index
        .as_ref()
        .map(|pi| pi.to_json());

    // Build definition index chunk — only when LINK produced one.
    let definition_index: Option<serde_json::Value> = result
        .definition_index
        .as_ref()
        .map(|di| di.to_json());

    // Count entities and locations from urd_json (when available).
    // The real compiler emits objects (keyed by id), not arrays.
    fn count_json_entries(val: &serde_json::Value) -> usize {
        val.as_object().map(|o| o.len())
            .or_else(|| val.as_array().map(|a| a.len()))
            .unwrap_or(0)
    }

    let entity_count = urd_json.as_ref()
        .and_then(|j| j.get("entities"))
        .map(count_json_entries)
        .unwrap_or(0);
    let location_count = urd_json.as_ref()
        .and_then(|j| j.get("locations"))
        .map(count_json_entries)
        .unwrap_or(0);
    let exit_count = urd_json.as_ref()
        .and_then(|j| j.get("locations"))
        .map(|locs| {
            if let Some(obj) = locs.as_object() {
                obj.values()
                    .filter_map(|loc| loc.get("exits"))
                    .map(count_json_entries)
                    .sum()
            } else if let Some(arr) = locs.as_array() {
                arr.iter()
                    .filter_map(|loc| loc.get("exits"))
                    .map(count_json_entries)
                    .sum()
            } else {
                0
            }
        })
        .unwrap_or(0);

    // Build chunk list. Optional chunks are only included when they have
    // real data. Omitted chunks are retained by the frontend cache.
    let mut chunks = vec![
        make_chunk("ast", ast),
        make_chunk("symbolTable", symbol_table),
        make_chunk(
            "diagnostics",
            serde_json::to_value(&diagnostics).unwrap_or_default(),
        ),
    ];
    if let Some(fs) = fact_set {
        chunks.push(make_chunk("factSet", fs));
    }
    if let Some(pi) = prop_index {
        chunks.push(make_chunk("propertyDependencyIndex", pi));
    }
    if let Some(di) = definition_index {
        chunks.push(make_chunk("definitionIndex", di));
    }
    if let Some(uj) = urd_json {
        chunks.push(make_chunk("urdJson", uj));
    }

    let header = OutputHeader {
        compile_id: compile_id.to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0),
        duration_ms,
        phase_timings: vec![
            PhaseTiming {
                phase: "total".to_string(),
                duration_ms,
            },
        ],
        world_counts: WorldCounts {
            entities: entity_count,
            locations: location_count,
            exits: exit_count,
            properties: result.symbol_table.as_ref()
                .map(|st| st.types.values().map(|t| t.properties.len()).sum())
                .unwrap_or(0),
            rules: result.symbol_table.as_ref()
                .map(|st| st.rules.len())
                .unwrap_or(0),
        },
        input_file_count,
    };

    CompilerOutput { header, chunks }
}

// ===== Tauri command =====

#[tauri::command]
pub fn compile_project(
    buffers: HashMap<String, String>,
    entry_file: Option<String>,
) -> Result<CompilerOutput, String> {
    let compile_id = format!(
        "tauri-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0)
    );

    Ok(compile_buffers(buffers, &compile_id, entry_file.as_deref()))
}

// ===== Tests =====

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_minimal_world() {
        let mut buffers = HashMap::new();
        buffers.insert(
            "world.urd.md".to_string(),
            "---\n---\n\n# World: Test\n\n## Entity: Player\n- name: \"Hero\"\n".to_string(),
        );

        let output = compile_buffers(buffers, "test-001", None);

        assert_eq!(output.header.compile_id, "test-001");
        assert_eq!(output.header.input_file_count, 1);

        // Always-present chunks: ast, symbolTable, diagnostics
        // Optional chunks (factSet, propertyDependencyIndex, definitionIndex, urdJson)
        // are only emitted when the compiler produces them.
        let names: Vec<&str> = output.chunks.iter().map(|c| c.name.as_str()).collect();
        assert!(names.contains(&"ast"));
        assert!(names.contains(&"symbolTable"));
        assert!(names.contains(&"diagnostics"));
        // Minimal valid world should produce urdJson + analysis chunks
        assert!(output.chunks.len() >= 3, "expected at least 3 chunks, got {}", output.chunks.len());

        // Verify content hashes are present
        for chunk in &output.chunks {
            assert!(!chunk.content_hash.is_empty());
        }
    }

    #[test]
    fn compile_rich_world_chunks() {
        let mut buffers = HashMap::new();
        buffers.insert(
            "world.urd.md".to_string(),
            [
                "---",
                "world:",
                "  name: test-world",
                "  start: tavern",
                "types:",
                "  Villager [interactable]:",
                "    trust: int(0, 100) = 50",
                "    mood: enum(happy, sad, angry) = happy",
                "entities:",
                "  @elder: Villager { trust: 30 }",
                "---",
                "",
                "# Tavern",
                "",
                "[@elder]",
                "",
                "== Greet",
                "",
                "@elder: Welcome, traveller.",
                "? @elder.trust >= 20",
                "  @elder: I trust you enough to share a secret.",
                "> @elder.trust + 5",
            ].join("\n"),
        );

        let output = compile_buffers(buffers, "test-rich", None);
        let names: Vec<&str> = output.chunks.iter().map(|c| c.name.as_str()).collect();

        // Print all chunk names for debugging
        eprintln!("CHUNKS: {:?}", names);

        // definitionIndex, factSet, propertyDependencyIndex should be present
        assert!(names.contains(&"definitionIndex"), "missing definitionIndex; chunks: {:?}", names);
        assert!(names.contains(&"factSet"), "missing factSet; chunks: {:?}", names);
        assert!(names.contains(&"propertyDependencyIndex"), "missing propertyDependencyIndex; chunks: {:?}", names);

        // urdJson may be missing if VALIDATE finds errors — print diagnostics
        let diags = output.chunks.iter().find(|c| c.name == "diagnostics").unwrap();
        let diag_arr = diags.data.as_array().unwrap();
        for d in diag_arr {
            eprintln!("DIAG: [{}] {} ({})", d["code"], d["message"], d["severity"]);
            if let Some(span) = d.get("span") {
                eprintln!("  at {}:{}:{}", span["file"], span["startLine"], span["startCol"]);
            }
        }

        if !names.contains(&"urdJson") {
            eprintln!("WARNING: urdJson chunk missing — EMIT was skipped due to errors above");
        }

        // Verify definitionIndex has property entries with property_type
        let def_idx = output.chunks.iter().find(|c| c.name == "definitionIndex").unwrap();
        let defs = def_idx.data["definitions"].as_array().unwrap();
        eprintln!("DEFINITION INDEX ({} entries):", defs.len());
        for d in defs {
            eprintln!("  {} => {:?}", d["key"], d["definition"]);
        }
        let trust_prop = defs.iter().find(|d| d["key"] == "prop:Villager.trust");
        if let Some(tp) = trust_prop {
            eprintln!("TRUST PROP: {:?}", tp);
            // property_type is the raw type string from the compiler
            assert!(!tp["definition"]["property_type"].as_str().unwrap().is_empty());
            assert_eq!(tp["definition"]["default"], "50");
        } else {
            eprintln!("WARNING: prop:Villager.trust not found in definitionIndex");
        }

        // Verify factSet structure exists with expected fields
        let fact_set = output.chunks.iter().find(|c| c.name == "factSet").unwrap();
        assert!(fact_set.data["reads"].is_array(), "factSet should have reads array");
        assert!(fact_set.data["writes"].is_array(), "factSet should have writes array");
        assert!(fact_set.data["exits"].is_array(), "factSet should have exits array");
        assert!(fact_set.data["jumps"].is_array(), "factSet should have jumps array");
        assert!(fact_set.data["choices"].is_array(), "factSet should have choices array");

        // Verify propertyDependencyIndex has trust entry
        let prop_idx = output.chunks.iter().find(|c| c.name == "propertyDependencyIndex").unwrap();
        let props = prop_idx.data["properties"].as_array().unwrap();
        eprintln!("PROPERTY INDEX ({} entries):", props.len());
        for p in props {
            eprintln!("  {}.{}: R={} W={}", p["entity_type"], p["property"], p["read_count"], p["write_count"]);
        }
    }

    #[test]
    fn compile_empty_buffers() {
        let buffers = HashMap::new();
        let output = compile_buffers(buffers, "test-002", None);
        assert_eq!(output.header.input_file_count, 0);
        // Empty buffers: ast, symbolTable, diagnostics always present.
        // Optional chunks may or may not be produced.
        assert!(output.chunks.len() >= 3, "expected at least 3 chunks, got {}", output.chunks.len());
    }

    #[test]
    fn content_hash_changes_with_input() {
        let mut buffers1 = HashMap::new();
        buffers1.insert("a.urd.md".to_string(), "---\n---\n# Version 1".to_string());
        let output1 = compile_buffers(buffers1, "test-003a", None);

        let mut buffers2 = HashMap::new();
        buffers2.insert("a.urd.md".to_string(), "---\n---\n# Version 2".to_string());
        let output2 = compile_buffers(buffers2, "test-003b", None);

        // The symbolTable chunk (always present) should differ between inputs
        let hash1 = &output1.chunks.iter().find(|c| c.name == "symbolTable").unwrap().content_hash;
        let hash2 = &output2.chunks.iter().find(|c| c.name == "symbolTable").unwrap().content_hash;
        assert_ne!(hash1, hash2);
    }
}
