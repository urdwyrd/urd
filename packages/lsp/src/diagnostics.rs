/// Compiler diagnostics → LSP diagnostics mapping and push.

use std::collections::HashMap;

use lsp_server::Connection;
use lsp_types::notification::Notification;
use lsp_types::Uri;

use crate::world_state::{self, WorldState};

/// Push diagnostics for all tracked files to the editor.
///
/// Groups diagnostics by file, sends a `publishDiagnostics` notification
/// per file, and clears diagnostics for files that no longer have errors.
pub fn push_diagnostics(connection: &Connection, state: &WorldState) {
    let result = match &state.result {
        Some(r) => r,
        None => return,
    };

    let entry_dir = match state.entry_dir() {
        Some(d) => d,
        None => return,
    };

    // Group compiler diagnostics by file
    let mut by_file: HashMap<String, Vec<lsp_types::Diagnostic>> = HashMap::new();
    for d in result.diagnostics.sorted() {
        let lsp_diag = to_lsp_diagnostic(d);
        by_file.entry(d.span.file.clone()).or_default().push(lsp_diag);
    }

    // Push per-file diagnostics
    for (file, diags) in &by_file {
        let uri = world_state::span_file_to_uri(file, &entry_dir);
        send_diagnostics(connection, uri, diags.clone());
    }

    // Clear diagnostics for previously tracked files that no longer have errors
    for tracked in &state.tracked_files {
        let filename = tracked
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_default();
        let tracked_str = tracked.to_string_lossy().replace('\\', "/");
        if !by_file.contains_key(&filename) && !by_file.contains_key(&tracked_str) {
            let uri = world_state::path_to_uri(tracked);
            send_diagnostics(connection, uri, vec![]);
        }
    }
}

fn send_diagnostics(connection: &Connection, uri: Uri, diagnostics: Vec<lsp_types::Diagnostic>) {
    let params = lsp_types::PublishDiagnosticsParams {
        uri,
        diagnostics,
        version: None,
    };
    let notification = lsp_server::Notification {
        method: lsp_types::notification::PublishDiagnostics::METHOD.to_string(),
        params: serde_json::to_value(params).unwrap(),
    };
    connection
        .sender
        .send(lsp_server::Message::Notification(notification))
        .ok();
}

fn to_lsp_diagnostic(d: &urd_compiler::diagnostics::Diagnostic) -> lsp_types::Diagnostic {
    let severity = match d.severity {
        urd_compiler::diagnostics::Severity::Error => lsp_types::DiagnosticSeverity::ERROR,
        urd_compiler::diagnostics::Severity::Warning => lsp_types::DiagnosticSeverity::WARNING,
        urd_compiler::diagnostics::Severity::Info => lsp_types::DiagnosticSeverity::INFORMATION,
    };

    let related_information = if d.related.is_empty() {
        None
    } else {
        Some(
            d.related
                .iter()
                .filter_map(|r| {
                    let uri_str = format!("file:///{}", r.span.file);
                    let uri: Uri = uri_str.parse().ok()?;
                    Some(lsp_types::DiagnosticRelatedInformation {
                        location: lsp_types::Location {
                            uri,
                            range: world_state::span_to_range(&r.span),
                        },
                        message: r.message.clone(),
                    })
                })
                .collect(),
        )
    };

    lsp_types::Diagnostic {
        range: world_state::span_to_range(&d.span),
        severity: Some(severity),
        code: Some(lsp_types::NumberOrString::String(d.code.clone())),
        source: Some("urd".to_string()),
        message: d.message.clone(),
        related_information,
        ..Default::default()
    }
}
