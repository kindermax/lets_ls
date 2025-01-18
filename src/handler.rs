use lsp_server::{Notification, Request, RequestId};
use lsp_types::CompletionParams;
use lsp_types::{
    lsif::DefinitionResultType, request::GotoTypeDefinitionParams, DidChangeTextDocumentParams,
    DidOpenTextDocumentParams, Location, Range,
};

use crate::state::State;
use crate::treesitter::{Command, Parser, PositionType};

#[derive(Debug)]
pub struct DefinitionResult {
    pub id: RequestId,
    pub value: DefinitionResultType,
}

#[derive(Debug)]
pub struct LSPCompletion {
    pub label: String,
    pub details: Option<String>,
    pub location: Option<LSPLocation>, // to support textEdit
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

// Construct a new URI from the current URI and the filename and return it if file exists.
// uri: current URI in format file://path/to/file
// filename: filename to append to the current URI, e.g. "lets.my.yaml"
fn go_to_def_uri(uri: &str, filename: &str) -> Option<String> {
    let parent = std::path::Path::new(uri.strip_prefix("file://")?).parent()?;
    let file = parent.join(filename);
    if file.exists() {
        return Some(format!("file://{}", file.to_str()?));
    }
    None
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

    let parser = Parser::new();

    if !matches!(parser.get_position_type(doc, &pos), PositionType::Mixins) {
        return None;
    }

    let filename = parser.extract_filename(doc, &pos)?;

    let uri = go_to_def_uri(uri, &filename);
    match uri {
        Some(uri) => {
            let result =
                DefinitionResultType::Scalar(lsp_types::lsif::LocationOrRangeId::Location(
                    Location::new(uri.parse().ok()?, Range::default()),
                ));
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
    let parser = Parser::new();
    let position_type = parser.get_position_type(doc, &position);

    let items = match position_type {
        PositionType::Depends => {
            let commands = parser.get_commands(doc);
            let current_command = parser.get_current_command(doc, &position)?;
            on_completion_depends(&current_command, &commands).ok()?
        }
        PositionType::Mixins => on_completion_mixins().ok()?,
        _ => vec![],
    };
    Some(LspResult::Completion(CompletionResult {
        id: req.id,
        list: items,
    }))
}

fn on_completion_depends(
    current_command: &Command,
    commands: &[Command],
) -> anyhow::Result<Vec<LSPCompletion>> {
    commands
        .iter()
        // TODO: do not complete already added commands to depends list
        .filter(|cmd| cmd.name != current_command.name)
        .map(|cmd| -> anyhow::Result<LSPCompletion> {
            Ok(LSPCompletion {
                label: cmd.name.clone(),
                details: None,
                location: None,
            })
        })
        .collect()
}

fn on_completion_mixins() -> anyhow::Result<Vec<LSPCompletion>> {
    // walk current dir or take word as dir if with /
    Ok(vec![])
}

#[cfg(test)]
mod tests {
    use super::*;
    use lsp_types::Position;
    use tempfile::NamedTempFile;

    #[test]
    fn test_go_to_def_filename() {
        let file = NamedTempFile::new().unwrap();
        let temppath = file.into_temp_path();
        let filename = temppath.file_name().unwrap().to_str().unwrap();
        let path = temppath.to_str().unwrap();

        let uri = format!("file://{path}");
        assert_eq!(go_to_def_uri(&uri, filename), Some(uri));
    }

    #[test]
    fn test_complete_depends_block_sequence() {
        let doc = r#"
shell: bash
commands:
  test:
    cmd: echo Test
    depends:
      -
  test2:
    cmd: echo Test2"#
            .trim();

        let parser = Parser::new();
        let position = Position::new(5, 7);
        let commands = parser.get_commands(doc);
        let command = parser
            .get_current_command(doc, &position)
            .expect("Command not found");
        let result = on_completion_depends(&command, &commands).expect("Completion failed");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].label, "test2");
        assert!(result[0].location.is_none());
    }

    #[test]
    fn test_complete_depends_flow_node() {
        let doc = r#"
shell: bash
commands:
  test:
    cmd: echo Test
    depends: []
  test2:
    cmd: echo Test2"#
            .trim();

        let position = Position::new(4, 14);
        let parser = Parser::new();
        let commands = parser.get_commands(doc);
        let command = parser
            .get_current_command(doc, &position)
            .expect("Command not found");
        let result = on_completion_depends(&command, &commands).expect("Completion failed");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].label, "test2");
        assert!(result[0].location.is_none());
    }
}
