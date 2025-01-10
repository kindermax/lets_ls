use lsp_server::{Notification, Request, RequestId};
use lsp_types::{CompletionParams, Position};
use lsp_types::{
    lsif::DefinitionResultType, request::GotoTypeDefinitionParams, DidChangeTextDocumentParams,
    DidOpenTextDocumentParams, Location, Range,
};

use crate::state::State;
use crate::treesitter::{extract_filename, get_commands, get_position_type, is_mixin_root_node, word_after_cursor, word_before_cursor, Command, PositionType};

#[derive(Debug)]
pub struct DefinitionResult {
    pub id: RequestId,
    pub value: DefinitionResultType,
}

#[derive(Debug)]
pub struct LSPCompletion {
    pub label: String,
    pub details: Option<String>,
    pub location: LSPLocation,
}

#[derive(Debug, Default)]
pub struct LSPLocation {
    pub uri: String,
    pub range: Range,
}

#[derive(Debug)]
pub struct CompletionResult {
    pub id: RequestId,
    pub list: Vec<LSPCompletion>,
}

#[derive(Debug)]
pub enum LspResult {
    OK,
    Definition(DefinitionResult),
    Completion(CompletionResult),
}

#[allow(non_snake_case)]
pub fn handle_didOpen(notf: Notification, state: &mut State) -> Option<LspResult> {
    let params: DidOpenTextDocumentParams = serde_json::from_value(notf.params).ok()?;
    state.add_document(
        params.text_document.uri.to_string(),
        params.text_document.text,
    );
    Some(LspResult::OK)
}

#[allow(non_snake_case)]
pub fn handle_didChange(notf: Notification, state: &mut State) -> Option<LspResult> {
    let params: DidChangeTextDocumentParams = serde_json::from_value(notf.params).ok()?;
    for change in params.content_changes {
        state.update_document(params.text_document.uri.to_string(), change.text);
    }
    Some(LspResult::OK)
}

fn go_to_def_filename(uri: &str, filename: &str) -> String {
    let parent = std::path::Path::new(uri).parent().unwrap();
    parent.join(filename).to_str().unwrap().to_string()
}

pub fn handle_definition(req: Request, state: &mut State) -> Option<LspResult> {
    let params: GotoTypeDefinitionParams = serde_json::from_value(req.params).ok()?;

    let uri = params
        .text_document_position_params
        .text_document
        .uri
        .as_str();

    let doc = state.get_document(uri)?;
    let pos = params.text_document_position_params.position;

    if !is_mixin_root_node(doc, &pos) {
        return None;
    }

    let filename = extract_filename(doc, &pos)?;

    let uri = go_to_def_filename(uri, &filename).parse().ok();
    match uri {
        Some(uri) => {
            let result = DefinitionResultType::Scalar(
                lsp_types::lsif::LocationOrRangeId::Location(Location::new(uri, Range::default())),
            );
            Some(LspResult::Definition(DefinitionResult {
                id: req.id,
                value: result,
            }))
        }
        None => None,
    }
}



pub fn handle_completion(req: Request, state: &mut State) -> Option<LspResult> {
    let params: CompletionParams = serde_json::from_value(req.params).ok()?;
    let uri = params.text_document_position.text_document.uri.as_str();
    let doc = state.get_document(uri)?;

    let position = params.text_document_position.position;
    let line = doc.lines().nth(position.line as usize)?;

    let items = match get_position_type(doc, position) {
        PositionType::Depends => {
            let commands = get_commands(doc);
            on_completion_depends(&commands, uri, line, position).ok()?
        },
        // TODO: mixins
        _ => return None,
    };
    Some(LspResult::Completion(CompletionResult {
        id: req.id,
        list: items,
    }))
}

fn on_completion_depends(commands: &Vec<Command>, uri: &str, line: &str, position: Position) -> anyhow::Result<Vec<LSPCompletion>> {
    let word = word_before_cursor(
        line,
        position.character as usize,
        |c: char| c.is_whitespace(),
    );
    let after = word_after_cursor(line, position.character as usize, |c| {
        c.is_whitespace()
    });

    commands
    .iter()
    .map(|cmd| -> anyhow::Result<LSPCompletion> {
        Ok(LSPCompletion {
            label: cmd.name.clone(),
            details: None,
            location: LSPLocation {
                uri: uri.to_string(),
                range: Range {
                    start: Position {
                        line: position.line,
                        character: position.character - u32::try_from(word.len())?,
                    },
                    end: Position {
                        line: position.line,
                        character: position.character + u32::try_from(after.len())?,
                    },
                },
            }
        })
    })
    .collect()
}