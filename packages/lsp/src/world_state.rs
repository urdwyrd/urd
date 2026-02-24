/// Compilation state management for the LSP server.
///
/// Tracks the entry file, latest compilation result, and stale-retained
/// indices that survive failed recompilations.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use lsp_types::Uri;
use urd_compiler::definition_index::DefinitionIndex;
use urd_compiler::facts::FactSet;
use urd_compiler::CompilationResult;

pub struct WorldState {
    /// Path to the entry file (first .urd.md opened).
    pub entry_path: Option<PathBuf>,
    /// Latest compilation result — always updated for diagnostics.
    pub result: Option<CompilationResult>,
    /// Latest DefinitionIndex — stale-retained when LINK fails.
    pub definition_index: Option<DefinitionIndex>,
    /// Parsed world JSON — stale-retained when EMIT fails.
    pub world_json: Option<serde_json::Value>,
    /// Files in the compilation unit (entry + imports).
    pub tracked_files: HashSet<PathBuf>,
}

impl WorldState {
    pub fn new() -> Self {
        Self {
            entry_path: None,
            result: None,
            definition_index: None,
            world_json: None,
            tracked_files: HashSet::new(),
        }
    }

    /// Recompile from the entry file and update state.
    ///
    /// - `result` is always replaced (for diagnostics).
    /// - `definition_index` is only replaced when LINK succeeds.
    /// - `world_json` is only replaced when EMIT succeeds.
    /// - `property_index` and `fact_set` are accessed via `result` directly.
    pub fn recompile(&mut self) {
        let entry = match &self.entry_path {
            Some(p) => p.to_string_lossy().to_string(),
            None => return,
        };

        let result = urd_compiler::compile(&entry);

        // Update stale-retained definition_index only when new data is available
        if let Some(ref idx) = result.definition_index {
            self.definition_index = Some(idx.clone());
        }

        // Update world_json only when EMIT succeeds
        if let Some(ref world_str) = result.world {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(world_str) {
                self.world_json = Some(parsed);
            }
        }

        // Always replace result (for diagnostics; also gives access to fact_set, property_index)
        self.result = Some(result);
    }

    /// Access the FactSet from the latest compilation result.
    pub fn fact_set(&self) -> Option<&FactSet> {
        self.result
            .as_ref()
            .and_then(|r| r.fact_set.as_ref())
    }

    /// Access the PropertyDependencyIndex from the latest compilation result.
    pub fn property_index(&self) -> Option<&urd_compiler::facts::PropertyDependencyIndex> {
        self.result
            .as_ref()
            .and_then(|r| r.property_index.as_ref())
    }

    /// The entry file's parent directory, for resolving relative span paths.
    pub fn entry_dir(&self) -> Option<PathBuf> {
        self.entry_path.as_ref().and_then(|p| p.parent()).map(|p| p.to_path_buf())
    }
}

// ── URI / path conversion ──

/// Convert an LSP URI to a filesystem path.
pub fn uri_to_path(uri: &Uri) -> PathBuf {
    let s = uri.as_str();
    // Strip file:/// prefix and decode
    if let Some(path_str) = s.strip_prefix("file:///") {
        // On Windows, file:///C:/path → C:/path
        // On Unix, file:///path → /path
        let decoded = percent_decode(path_str);
        if cfg!(windows) {
            PathBuf::from(decoded)
        } else {
            PathBuf::from(format!("/{}", decoded))
        }
    } else if let Some(path_str) = s.strip_prefix("file://") {
        PathBuf::from(percent_decode(path_str))
    } else {
        PathBuf::from(s)
    }
}

/// Convert a filesystem path to an LSP URI.
pub fn path_to_uri(path: &Path) -> Uri {
    let normalised = path.to_string_lossy().replace('\\', "/");
    let uri_str = if normalised.starts_with('/') {
        format!("file://{}", normalised)
    } else {
        format!("file:///{}", normalised)
    };
    uri_str.parse::<Uri>().unwrap()
}

/// Resolve a compiler span file path to an absolute path, then to a URI.
///
/// Compiler spans use forward-slash relative paths (e.g. `locked-garden.urd.md`).
/// We join with the entry file's parent directory to get an absolute path.
pub fn span_file_to_uri(span_file: &str, entry_dir: &Path) -> Uri {
    let absolute = entry_dir.join(span_file);
    path_to_uri(&absolute)
}

/// Convert a compiler Span (1-indexed) to an LSP Range (0-indexed).
pub fn span_to_range(span: &urd_compiler::span::Span) -> lsp_types::Range {
    lsp_types::Range {
        start: lsp_types::Position {
            line: span.start_line.saturating_sub(1),
            character: span.start_col.saturating_sub(1),
        },
        end: lsp_types::Position {
            line: span.end_line.saturating_sub(1),
            character: span.end_col.saturating_sub(1),
        },
    }
}

/// Convert a compiler Span to an LSP Location (with URI).
pub fn span_to_location(
    span: &urd_compiler::span::Span,
    entry_dir: &Path,
) -> lsp_types::Location {
    lsp_types::Location {
        uri: span_file_to_uri(&span.file, entry_dir),
        range: span_to_range(span),
    }
}

/// Simple percent-decoding for URI path segments.
fn percent_decode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                result.push(byte as char);
            } else {
                result.push('%');
                result.push_str(&hex);
            }
        } else {
            result.push(c);
        }
    }
    result
}
