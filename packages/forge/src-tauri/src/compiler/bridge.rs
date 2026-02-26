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
) -> CompilerOutput {
    let start = std::time::Instant::now();

    // Determine entry file (first .urd.md alphabetically, or first file)
    let entry_filename = buffers
        .keys()
        .filter(|k| k.ends_with(".urd.md"))
        .min()
        .or_else(|| buffers.keys().next())
        .cloned()
        .unwrap_or_else(|| "unknown.urd.md".to_string());

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

    // Build urdJson chunk from world output
    let urd_json: serde_json::Value = result
        .world
        .as_ref()
        .and_then(|w| serde_json::from_str(w).ok())
        .unwrap_or(serde_json::json!({"entities": [], "locations": []}));

    // Build symbol table chunk
    let symbol_table = serde_json::json!({
        "entries": result.symbol_table.as_ref().map(|_st| {
            // Serialise minimal symbol table entries for the frontend
            // Full serialisation deferred until views consume the data
            serde_json::json!([])
        }).unwrap_or(serde_json::json!([]))
    });

    // Build fact set chunk
    let fact_set = serde_json::json!({
        "facts": result.fact_set.as_ref().map(|_| {
            serde_json::json!([])
        }).unwrap_or(serde_json::json!([]))
    });

    // Build AST chunk (placeholder — full AST serialisation deferred)
    let ast = serde_json::json!({ "nodes": [] });

    // Build property dependency index chunk
    let prop_index = serde_json::json!({
        "dependencies": result.property_index.as_ref().map(|_| {
            serde_json::json!([])
        }).unwrap_or(serde_json::json!([]))
    });

    // Count entities and locations from urd_json
    let entity_count = urd_json
        .get("entities")
        .and_then(|e| e.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    let location_count = urd_json
        .get("locations")
        .and_then(|l| l.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    let exit_count = urd_json
        .get("locations")
        .and_then(|l| l.as_array())
        .map(|locs| {
            locs.iter()
                .filter_map(|loc| loc.get("exits"))
                .filter_map(|e| e.as_array())
                .map(|a| a.len())
                .sum()
        })
        .unwrap_or(0);

    let chunks = vec![
        make_chunk("ast", ast),
        make_chunk("symbolTable", symbol_table),
        make_chunk("factSet", fact_set),
        make_chunk("propertyDependencyIndex", prop_index),
        make_chunk("urdJson", urd_json),
        make_chunk(
            "diagnostics",
            serde_json::to_value(&diagnostics).unwrap_or_default(),
        ),
    ];

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
            properties: 0,
            rules: 0,
        },
        input_file_count,
    };

    CompilerOutput { header, chunks }
}

// ===== Tauri command =====

#[tauri::command]
pub fn compile_project(
    buffers: HashMap<String, String>,
) -> Result<CompilerOutput, String> {
    let compile_id = format!(
        "tauri-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0)
    );

    Ok(compile_buffers(buffers, &compile_id))
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

        let output = compile_buffers(buffers, "test-001");

        assert_eq!(output.header.compile_id, "test-001");
        assert_eq!(output.header.input_file_count, 1);
        assert_eq!(output.chunks.len(), 6);

        // Verify chunk names
        let names: Vec<&str> = output.chunks.iter().map(|c| c.name.as_str()).collect();
        assert!(names.contains(&"ast"));
        assert!(names.contains(&"symbolTable"));
        assert!(names.contains(&"urdJson"));
        assert!(names.contains(&"diagnostics"));

        // Verify content hashes are present
        for chunk in &output.chunks {
            assert!(!chunk.content_hash.is_empty());
        }
    }

    #[test]
    fn compile_empty_buffers() {
        let buffers = HashMap::new();
        let output = compile_buffers(buffers, "test-002");
        assert_eq!(output.header.input_file_count, 0);
        assert_eq!(output.chunks.len(), 6);
    }

    #[test]
    fn content_hash_changes_with_input() {
        let mut buffers1 = HashMap::new();
        buffers1.insert("a.urd.md".to_string(), "---\n---\n# Version 1".to_string());
        let output1 = compile_buffers(buffers1, "test-003a");

        let mut buffers2 = HashMap::new();
        buffers2.insert("a.urd.md".to_string(), "---\n---\n# Version 2".to_string());
        let output2 = compile_buffers(buffers2, "test-003b");

        // At least the urdJson chunk should have a different hash
        let hash1 = &output1.chunks.iter().find(|c| c.name == "urdJson").unwrap().content_hash;
        let hash2 = &output2.chunks.iter().find(|c| c.name == "urdJson").unwrap().content_hash;
        assert_ne!(hash1, hash2);
    }
}
