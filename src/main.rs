use std::error::Error;

use lsp_server::Connection;
use lsp_types::{ClientCapabilities, InitializeParams, ServerCapabilities};

fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    eprintln!("Lets LSP server starting");

    let (connection, io_threads) = Connection::stdio();
    let (id, params) = connection.initialize_start()?;
    let init_params: InitializeParams = serde_json::from_value(params).unwrap();

    let client_capabilities: ClientCapabilities = init_params.capabilities;
    let server_capabilities = ServerCapabilities {
        text_document_sync: Some(lsp_types::TextDocumentSyncCapability::Kind(
            lsp_types::TextDocumentSyncKind::FULL,
        )),
        hover_provider: Some(lsp_types::HoverProviderCapability::Simple(true)),
        definition_provider: Some(lsp_types::OneOf::Left(true)),
        ..ServerCapabilities::default()
    };

    let initialize_data = serde_json::json!({
        "capabilities": server_capabilities,
        "serverInfo": {
            "name": "lets-ls",
            "version": "0.1.0"
        }
    });

    connection.initialize_finish(id, initialize_data)?;

    Ok(())
}
