use lsp_server::Message;

use crate::handler;

pub fn definition_response(result: handler::DefinitionResult) -> anyhow::Result<Message> {
    Ok(Message::Response(lsp_server::Response {
        id: result.id,
        result: serde_json::to_value(result.value).ok(),
        error: None,
    }))
}

pub fn completion_response(result: handler::CompletionResult) -> Message {
    Message::Response(lsp_server::Response {
        id: result.id,
        result: serde_json::to_value(lsp_types::CompletionList {
            items: result
                .list
                .iter()
                .map(|c| {
                    let mut item = lsp_types::CompletionItem {
                        label: c.label.clone(),
                        kind: Some(lsp_types::CompletionItemKind::KEYWORD),
                        ..Default::default()
                    };

                    if let Some(documentation) = c.details.clone() {
                        item.documentation =
                            Some(lsp_types::Documentation::MarkupContent(lsp_types::MarkupContent {
                                kind: lsp_types::MarkupKind::Markdown,
                                value: documentation.clone(),
                            }));
                    }

                    item
                })
                .collect(),
            is_incomplete: false,
        })
        .ok(),
        error: None,
    })
}