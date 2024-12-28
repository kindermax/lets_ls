use lsp_server::{Notification, Request, RequestId};
use lsp_types::{
    lsif::DefinitionResultType, request::GotoTypeDefinitionParams, DidChangeTextDocumentParams,
    DidOpenTextDocumentParams, Location, Range,
};

use crate::state::State;
use crate::treesitter::{extract_filename, is_mixin_root_node};

#[derive(Debug)]
pub struct DefinitionResult {
    pub id: RequestId,
    pub value: DefinitionResultType,
}

#[derive(Debug)]
pub enum LspResult {
    OK,
    Definition(DefinitionResult),
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
