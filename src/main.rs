use std::error::Error;

use lsp_server::{Connection, Message};
use lsp_types::ServerCapabilities;

use crate::handler::{
    handle_completion, handle_definition, handle_didChange, handle_didOpen, LspResult,
};
use crate::responses::{completion_response, definition_response};
use crate::state::State;

pub mod handler;
pub mod responses;
pub mod state;
pub mod treesitter;

fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    eprintln!("Lets LSP server starting");

    let (connection, io_threads) = Connection::stdio();
    let (id, _) = connection.initialize_start()?;

    let server_capabilities = ServerCapabilities {
        text_document_sync: Some(lsp_types::TextDocumentSyncCapability::Kind(
            lsp_types::TextDocumentSyncKind::FULL,
        )),
        hover_provider: Some(lsp_types::HoverProviderCapability::Simple(true)),
        definition_provider: Some(lsp_types::OneOf::Left(true)),
        completion_provider: Some(lsp_types::CompletionOptions {
            resolve_provider: Some(false),
            trigger_characters: Some(vec![
                ".".to_string(),
                ":".to_string(),
                " ".to_string(),
                "$".to_string(),
                "- ".to_string(),
                "[".to_string(),
            ]),
            work_done_progress_options: lsp_types::WorkDoneProgressOptions {
                work_done_progress: None,
            },
            all_commit_characters: None,
            completion_item: None,
        }),
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

    let mut state = State::new();

    for msg in &connection.receiver {
        let result: Option<LspResult> = match msg {
            Message::Request(req) => {
                eprintln!("--> Request: {:?}", req.method);
                match req.method.as_str() {
                    "textDocument/definition" => handle_definition(req, &mut state),
                    "textDocument/completion" => handle_completion(req, &mut state),
                    "shutdown" => {
                        connection.sender.send(
                            Message::Response(lsp_server::Response::new_ok(req.id, ()))
                        )?;
                        break;
                    },
                    _ => None,
                }
            }
            Message::Notification(notf) => {
                eprintln!("--> Notification: {:?}", notf.method);
                match notf.method.as_str() {
                    "textDocument/didOpen" => handle_didOpen(notf, &mut state),
                    "textDocument/didChange" => handle_didChange(notf, &mut state),
                    _ => None,
                }
            }
            _ => {
                eprintln!("--> Other message type: {:?}", msg);
                None
            }
        };
        eprintln!("<-- LspResult: {:?}", result);
        if let Some(result) = result {
            match result {
                LspResult::OK => (),
                LspResult::Definition(result) => {
                    connection.sender.send(definition_response(result)?)?
                }
                LspResult::Completion(result) => {
                    connection.sender.send(completion_response(result))?
                }
            }
        }
    }

    io_threads.join()?;

    eprintln!("Lets LSP server shutting down");

    Ok(())
}
