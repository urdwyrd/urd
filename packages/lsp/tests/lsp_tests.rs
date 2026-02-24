/// Mock-client integration tests for the Urd LSP server.
///
/// Uses `Connection::memory()` for in-memory client ↔ server transport.
/// The server runs in a background thread; the test acts as the LSP client.

use std::time::Duration;

use lsp_server::{Connection, Message, Notification, Request, Response};
use lsp_types::*;
use serde_json::json;

fn fixture_path(name: &str) -> String {
    let base = env!("CARGO_MANIFEST_DIR");
    // Go up to packages/, then into compiler/tests/fixtures/
    format!("{}/../compiler/tests/fixtures/{}", base, name)
}

fn fixture_uri(name: &str) -> Uri {
    let path = fixture_path(name);
    let normalised = path.replace('\\', "/");
    let uri_str = if normalised.starts_with('/') {
        format!("file://{}", normalised)
    } else {
        format!("file:///{}", normalised)
    };
    uri_str.parse::<Uri>().unwrap()
}

/// Set up an in-memory LSP connection. Returns (client_conn, server_thread).
///
/// The client must complete the initialize handshake before sending other messages.
fn setup() -> (Connection, std::thread::JoinHandle<()>) {
    let (server_conn, client_conn) = Connection::memory();

    let server_thread = std::thread::spawn(move || {
        urd_lsp::run_server(server_conn);
    });

    (client_conn, server_thread)
}

/// Send the initialize request and initialized notification from the client side.
fn initialize(client: &Connection) -> InitializeResult {
    let init_params = InitializeParams {
        capabilities: ClientCapabilities::default(),
        ..Default::default()
    };

    // Send initialize request
    let req = Request {
        id: 1.into(),
        method: "initialize".to_string(),
        params: serde_json::to_value(init_params).unwrap(),
    };
    client.sender.send(Message::Request(req)).unwrap();

    // Receive initialize response
    let resp = match client.receiver.recv_timeout(Duration::from_secs(5)).unwrap() {
        Message::Response(r) => r,
        other => panic!("Expected Response, got {:?}", other),
    };

    let result: InitializeResult = serde_json::from_value(resp.result.unwrap()).unwrap();

    // Send initialized notification
    let not = Notification {
        method: "initialized".to_string(),
        params: json!({}),
    };
    client.sender.send(Message::Notification(not)).unwrap();

    result
}

/// Send a didOpen notification for a fixture file.
fn send_did_open(client: &Connection, fixture: &str) {
    let path = fixture_path(fixture);
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Cannot read fixture {}: {}", path, e));
    let uri = fixture_uri(fixture);
    let params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri,
            language_id: "urd".to_string(),
            version: 0,
            text: source,
        },
    };
    client
        .sender
        .send(Message::Notification(Notification {
            method: "textDocument/didOpen".to_string(),
            params: serde_json::to_value(params).unwrap(),
        }))
        .unwrap();
}

/// Send a didSave notification for a fixture file.
fn send_did_save(client: &Connection, fixture: &str) {
    let uri = fixture_uri(fixture);
    let params = DidSaveTextDocumentParams {
        text_document: TextDocumentIdentifier { uri },
        text: None,
    };
    client
        .sender
        .send(Message::Notification(Notification {
            method: "textDocument/didSave".to_string(),
            params: serde_json::to_value(params).unwrap(),
        }))
        .unwrap();
}

/// Send a shutdown request and exit notification.
fn shutdown(client: &Connection) {
    let req = Request {
        id: 999.into(),
        method: "shutdown".to_string(),
        params: json!(null),
    };
    client.sender.send(Message::Request(req)).unwrap();

    // Wait for shutdown response
    match client.receiver.recv_timeout(Duration::from_secs(5)) {
        Ok(Message::Response(_)) => {}
        other => panic!("Expected shutdown response, got {:?}", other),
    }

    let not = Notification {
        method: "exit".to_string(),
        params: json!(null),
    };
    client.sender.send(Message::Notification(not)).unwrap();
}

/// Receive a publishDiagnostics notification, with timeout.
fn recv_diagnostics(client: &Connection) -> PublishDiagnosticsParams {
    let timeout = Duration::from_secs(10);
    loop {
        match client.receiver.recv_timeout(timeout) {
            Ok(Message::Notification(not)) => {
                if not.method == "textDocument/publishDiagnostics" {
                    return serde_json::from_value(not.params).unwrap();
                }
                // Skip other notifications
            }
            Ok(other) => panic!("Expected notification, got {:?}", other),
            Err(_) => panic!("Timed out waiting for publishDiagnostics"),
        }
    }
}

/// Send a textDocument/definition request and return the response.
fn send_definition(client: &Connection, fixture: &str, line: u32, character: u32) -> Response {
    let uri = fixture_uri(fixture);
    let params = GotoDefinitionParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri },
            position: Position { line, character },
        },
        work_done_progress_params: Default::default(),
        partial_result_params: Default::default(),
    };
    let req = Request {
        id: 10.into(),
        method: "textDocument/definition".to_string(),
        params: serde_json::to_value(params).unwrap(),
    };
    client.sender.send(Message::Request(req)).unwrap();

    match client.receiver.recv_timeout(Duration::from_secs(5)).unwrap() {
        Message::Response(r) => r,
        other => panic!("Expected Response, got {:?}", other),
    }
}

/// Send a textDocument/hover request and return the response.
fn send_hover(client: &Connection, fixture: &str, line: u32, character: u32) -> Response {
    let uri = fixture_uri(fixture);
    let params = HoverParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri },
            position: Position { line, character },
        },
        work_done_progress_params: Default::default(),
    };
    let req = Request {
        id: 20.into(),
        method: "textDocument/hover".to_string(),
        params: serde_json::to_value(params).unwrap(),
    };
    client.sender.send(Message::Request(req)).unwrap();

    match client.receiver.recv_timeout(Duration::from_secs(5)).unwrap() {
        Message::Response(r) => r,
        other => panic!("Expected Response, got {:?}", other),
    }
}

/// Send a textDocument/completion request and return the response.
fn send_completion(client: &Connection, fixture: &str, line: u32, character: u32) -> Response {
    let uri = fixture_uri(fixture);
    let params = CompletionParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri },
            position: Position { line, character },
        },
        work_done_progress_params: Default::default(),
        partial_result_params: Default::default(),
        context: None,
    };
    let req = Request {
        id: 30.into(),
        method: "textDocument/completion".to_string(),
        params: serde_json::to_value(params).unwrap(),
    };
    client.sender.send(Message::Request(req)).unwrap();

    match client.receiver.recv_timeout(Duration::from_secs(5)).unwrap() {
        Message::Response(r) => r,
        other => panic!("Expected Response, got {:?}", other),
    }
}

// ── Tests ──

#[test]
fn lsp_initialize() {
    let (client, thread) = setup();
    let result = initialize(&client);

    // Verify server capabilities
    assert!(result.capabilities.definition_provider.is_some());
    assert!(result.capabilities.hover_provider.is_some());
    assert!(result.capabilities.completion_provider.is_some());

    shutdown(&client);
    thread.join().unwrap();
}

#[test]
fn lsp_diagnostics_on_save() {
    let (client, thread) = setup();
    initialize(&client);

    send_did_open(&client, "locked-garden.urd.md");
    let diags = recv_diagnostics(&client);

    // Locked garden is a clean fixture — should have no errors
    // (may have warnings, but no errors)
    let errors: Vec<_> = diags
        .diagnostics
        .iter()
        .filter(|d| d.severity == Some(DiagnosticSeverity::ERROR))
        .collect();
    assert!(
        errors.is_empty(),
        "Expected no errors for locked-garden, got: {:?}",
        errors
    );

    shutdown(&client);
    thread.join().unwrap();
}

#[test]
fn lsp_diagnostics_on_did_save() {
    let (client, thread) = setup();
    initialize(&client);

    // Open sets the entry path, then save triggers recompile
    send_did_open(&client, "locked-garden.urd.md");
    let _diags = recv_diagnostics(&client);

    // Save should trigger a fresh recompile + diagnostic push
    send_did_save(&client, "locked-garden.urd.md");
    let diags = recv_diagnostics(&client);

    let errors: Vec<_> = diags
        .diagnostics
        .iter()
        .filter(|d| d.severity == Some(DiagnosticSeverity::ERROR))
        .collect();
    assert!(
        errors.is_empty(),
        "Expected no errors after save, got: {:?}",
        errors
    );

    shutdown(&client);
    thread.join().unwrap();
}

#[test]
fn lsp_diagnostics_error_world() {
    let (client, thread) = setup();
    initialize(&client);

    // Use a negative fixture that produces errors
    send_did_open(&client, "negative-unreachable-location.urd.md");
    let diags = recv_diagnostics(&client);

    // Should contain at least one warning (URD430 unreachable location)
    assert!(
        !diags.diagnostics.is_empty(),
        "Expected diagnostics for negative fixture"
    );

    shutdown(&client);
    thread.join().unwrap();
}

#[test]
fn lsp_goto_entity() {
    let (client, thread) = setup();
    initialize(&client);

    send_did_open(&client, "locked-garden.urd.md");
    let _diags = recv_diagnostics(&client);

    // Line 28 (0-indexed): "[@warden, @iron_key]"
    // @warden starts at col 1
    let resp = send_definition(&client, "locked-garden.urd.md", 28, 3);

    // Should return a location (not null)
    assert!(
        resp.result.is_some(),
        "Expected definition result for @warden"
    );
    let result = resp.result.unwrap();
    // Should not be null
    assert!(
        !result.is_null(),
        "Expected non-null definition for @warden, got null"
    );

    shutdown(&client);
    thread.join().unwrap();
}

#[test]
fn lsp_goto_section() {
    let (client, thread) = setup();
    initialize(&client);

    send_did_open(&client, "locked-garden.urd.md");
    let _diags = recv_diagnostics(&client);

    // Line 41 (0-indexed): "  -> greet"
    // "-> greet" — the "greet" target
    let resp = send_definition(&client, "locked-garden.urd.md", 41, 6);

    assert!(
        resp.result.is_some(),
        "Expected definition result for -> greet"
    );

    shutdown(&client);
    thread.join().unwrap();
}

#[test]
fn lsp_goto_not_found() {
    let (client, thread) = setup();
    initialize(&client);

    send_did_open(&client, "locked-garden.urd.md");
    let _diags = recv_diagnostics(&client);

    // Line 2 (0-indexed): "A stone archway..." — plain prose, no identifier
    let resp = send_definition(&client, "locked-garden.urd.md", 26, 5);

    // Should return null (no definition found)
    let result = resp.result.unwrap();
    assert!(
        result.is_null(),
        "Expected null for prose line, got {:?}",
        result
    );

    shutdown(&client);
    thread.join().unwrap();
}

#[test]
fn lsp_hover_entity() {
    let (client, thread) = setup();
    initialize(&client);

    send_did_open(&client, "locked-garden.urd.md");
    let _diags = recv_diagnostics(&client);

    // Line 28: "[@warden, @iron_key]"
    let resp = send_hover(&client, "locked-garden.urd.md", 28, 3);

    assert!(resp.result.is_some(), "Expected hover result for @warden");
    let result = resp.result.unwrap();
    assert!(!result.is_null(), "Expected non-null hover for @warden");

    // Parse hover content
    let hover: Hover = serde_json::from_value(result).unwrap();
    match hover.contents {
        HoverContents::Markup(markup) => {
            assert!(
                markup.value.contains("Character"),
                "Hover should mention Character type, got: {}",
                markup.value
            );
        }
        other => panic!("Expected Markup hover, got {:?}", other),
    }

    shutdown(&client);
    thread.join().unwrap();
}

#[test]
fn lsp_hover_property() {
    let (client, thread) = setup();
    initialize(&client);

    send_did_open(&client, "locked-garden.urd.md");
    let _diags = recv_diagnostics(&client);

    // Line 50: "  ? @warden.trust >= 3"
    // cursor on "trust" (col ~14)
    let resp = send_hover(&client, "locked-garden.urd.md", 50, 14);

    assert!(
        resp.result.is_some(),
        "Expected hover result for @warden.trust"
    );
    let result = resp.result.unwrap();
    if !result.is_null() {
        let hover: Hover = serde_json::from_value(result).unwrap();
        match hover.contents {
            HoverContents::Markup(markup) => {
                assert!(
                    markup.value.contains("trust"),
                    "Hover should mention trust, got: {}",
                    markup.value
                );
            }
            other => panic!("Expected Markup hover, got {:?}", other),
        }
    }

    shutdown(&client);
    thread.join().unwrap();
}

#[test]
fn lsp_hover_section() {
    let (client, thread) = setup();
    initialize(&client);

    send_did_open(&client, "locked-garden.urd.md");
    let _diags = recv_diagnostics(&client);

    // Line 34 (0-indexed): "== greet"
    let resp = send_hover(&client, "locked-garden.urd.md", 34, 5);

    assert!(
        resp.result.is_some(),
        "Expected hover result for == greet"
    );
    let result = resp.result.unwrap();
    if !result.is_null() {
        let hover: Hover = serde_json::from_value(result).unwrap();
        match hover.contents {
            HoverContents::Markup(markup) => {
                assert!(
                    markup.value.contains("Section"),
                    "Hover should mention Section, got: {}",
                    markup.value
                );
            }
            other => panic!("Expected Markup hover, got {:?}", other),
        }
    }

    shutdown(&client);
    thread.join().unwrap();
}

#[test]
fn lsp_autocomplete_section() {
    let (client, thread) = setup();
    initialize(&client);

    send_did_open(&client, "locked-garden.urd.md");
    let _diags = recv_diagnostics(&client);

    // Line 41: "  -> greet" — simulate cursor after "-> "
    let resp = send_completion(&client, "locked-garden.urd.md", 41, 5);

    assert!(
        resp.result.is_some(),
        "Expected completion result"
    );
    let result = resp.result.unwrap();
    if !result.is_null() {
        let items: CompletionResponse = serde_json::from_value(result).unwrap();
        match items {
            CompletionResponse::Array(items) => {
                assert!(
                    !items.is_empty(),
                    "Expected non-empty completion list for -> target"
                );
                // Should contain section names like "greet", "explore", "revelation"
                let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
                assert!(
                    labels.contains(&"greet"),
                    "Expected 'greet' in completions, got: {:?}",
                    labels
                );
            }
            _ => panic!("Expected Array completion response"),
        }
    }

    shutdown(&client);
    thread.join().unwrap();
}

#[test]
fn lsp_latency_sunken_citadel() {
    let (client, thread) = setup();
    initialize(&client);

    let start = std::time::Instant::now();
    send_did_open(&client, "sunken-citadel.urd.md");
    let _diags = recv_diagnostics(&client);
    let elapsed = start.elapsed();

    assert!(
        elapsed < Duration::from_millis(200),
        "Sunken Citadel recompile + diagnostic push took {:?}, expected < 200ms",
        elapsed
    );

    shutdown(&client);
    thread.join().unwrap();
}

// ── Import boundary test ──

#[test]
fn lsp_crate_does_not_import_ast_modules() {
    let prohibited = [
        "urd_compiler::ast",
        "urd_compiler::parse",
        "urd_compiler::link",
        "urd_compiler::validate",
        "urd_compiler::emit",
        "urd_compiler::symbol_table",
        "urd_compiler::graph",
    ];
    let lsp_src = format!("{}/src", env!("CARGO_MANIFEST_DIR"));
    for entry in glob::glob(&format!("{}/**/*.rs", lsp_src)).unwrap() {
        let path = entry.unwrap();
        let source = std::fs::read_to_string(&path).unwrap();
        for import in &prohibited {
            assert!(
                !source.contains(import),
                "LSP source {:?} contains prohibited import: {}",
                path, import,
            );
        }
    }
}
