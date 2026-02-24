/// LSP server capability registration.

use lsp_types::*;

pub fn server_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Options(
            TextDocumentSyncOptions {
                open_close: Some(true),
                save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                    include_text: Some(false),
                })),
                change: Some(TextDocumentSyncKind::NONE),
                ..Default::default()
            },
        )),
        definition_provider: Some(OneOf::Left(true)),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        completion_provider: Some(CompletionOptions {
            trigger_characters: Some(vec![
                "@".to_string(),
                ".".to_string(),
                ">".to_string(),
            ]),
            ..Default::default()
        }),
        ..Default::default()
    }
}
