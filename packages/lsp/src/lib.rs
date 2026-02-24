/// Urd Language Server — embeds the compiler with real-time diagnostics,
/// go-to-definition, hover, and autocomplete.
///
/// Communicates via stdin/stdout using the Language Server Protocol.
/// Synchronous, single-threaded, recompile-on-save.

pub mod capabilities;
pub mod completion;
pub mod cursor;
pub mod definition;
pub mod diagnostics;
pub mod hover;
pub mod world_state;

use lsp_server::{Connection, Message};
use lsp_types::InitializeParams;

/// Run the LSP server on the given connection.
///
/// Performs the initialize handshake, then enters the main message loop.
/// Returns when the client sends a shutdown request.
///
/// Exposed as a public function for mock-client testing via `Connection::memory()`.
pub fn run_server(connection: Connection) {
    let server_capabilities = capabilities::server_capabilities();
    let init_params = connection
        .initialize(serde_json::to_value(server_capabilities).unwrap())
        .unwrap();
    let _params: InitializeParams = serde_json::from_value(init_params).unwrap();

    let mut state = world_state::WorldState::new();
    main_loop(&connection, &mut state);
}

fn main_loop(connection: &Connection, state: &mut world_state::WorldState) {
    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req).unwrap() {
                    break;
                }
                handle_request(connection, state, req);
            }
            Message::Notification(not) => {
                handle_notification(connection, state, not);
            }
            Message::Response(_) => {}
        }
    }
}

fn handle_request(
    connection: &Connection,
    state: &world_state::WorldState,
    req: lsp_server::Request,
) {
    match req.method.as_str() {
        "textDocument/definition" => definition::handle(connection, state, req),
        "textDocument/hover" => hover::handle(connection, state, req),
        "textDocument/completion" => completion::handle(connection, state, req),
        _ => {
            let resp = lsp_server::Response::new_err(
                req.id,
                lsp_server::ErrorCode::MethodNotFound as i32,
                "Method not found".to_string(),
            );
            connection
                .sender
                .send(Message::Response(resp))
                .ok();
        }
    }
}

fn handle_notification(
    connection: &Connection,
    state: &mut world_state::WorldState,
    not: lsp_server::Notification,
) {
    match not.method.as_str() {
        "textDocument/didOpen" => {
            if let Ok(params) =
                serde_json::from_value::<lsp_types::DidOpenTextDocumentParams>(not.params)
            {
                let path = world_state::uri_to_path(&params.text_document.uri);
                if state.entry_path.is_none() {
                    state.entry_path = Some(path);
                    state.recompile();
                    diagnostics::push_diagnostics(connection, state);
                }
            }
        }
        "textDocument/didSave" => {
            if let Ok(params) =
                serde_json::from_value::<lsp_types::DidSaveTextDocumentParams>(not.params)
            {
                let path = world_state::uri_to_path(&params.text_document.uri);
                if state.entry_path.is_none() {
                    state.entry_path = Some(path);
                }
                state.recompile();
                diagnostics::push_diagnostics(connection, state);
            }
        }
        _ => {}
    }
}
