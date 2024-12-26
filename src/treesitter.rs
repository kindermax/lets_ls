use streaming_iterator::StreamingIterator;
use tree_sitter::{Node, Parser, Query, QueryCursor};

fn is_cursor_within_node(node: &Node, pos: &lsp_types::Position) -> bool {
    let start_point = node.start_position();
    let end_point = node.end_position();

  pos.line as usize >= start_point.row && pos.line as usize <= end_point.row &&
         pos.character as usize >= start_point.column &&
         pos.character as usize <= end_point.column
}

fn is_cursor_at_line(node: &Node, pos: &lsp_types::Position) -> bool {
    let start_point = node.start_position();
    let end_point = node.end_position();
    pos.line as usize == start_point.row && pos.line as usize == end_point.row
}

fn get_node_text<'a>(node: &Node, text: &'a str) -> Option<&'a str> {
    if let Ok(value) = node.utf8_text(text.as_bytes()) {
        return Some(value);
    }
    None
}

pub fn extract_filename(text: &str, pos: &lsp_types::Position) -> Option<String> {
    let mut parser = Parser::new();
    let language = tree_sitter_yaml::LANGUAGE.into();
    parser
        .set_language(&language)
        .expect("could not load yaml language");

    let tree = parser.parse(text, None).expect("could not parse text");
    let root = tree.root_node();

    let query = r#"
    (block_mapping_pair
        key: (flow_node) @key
        value: (block_node
            (block_sequence
                (block_sequence_item
                    (flow_node) @value)))
        (#eq? @key "mixins")
    )
    "#;

    let query = Query::new(&language, query).expect("could not create query");

    let mut cursor_qry = QueryCursor::new();

    let mut matches = cursor_qry.matches(&query, root, text.as_bytes());

    while let Some(m) = matches.next() {
        let found = m.captures.iter().find(|c| {
            if let Some(parent) = c.node.parent() {
                return parent.kind() == "block_sequence_item"
                    && is_cursor_at_line(&c.node, pos);
            }
            false
        });
        if let Some(found) = found {
            return get_node_text(&found.node, text).map(|s| s.to_string());
        }
    }
    None

}

pub fn is_mixin_root_node(text: &str, pos: &lsp_types::Position) -> bool {
    let mut parser = Parser::new();
    let language = tree_sitter_yaml::LANGUAGE.into();
    parser
        .set_language(&language)
        .expect("could not load yaml language");

    let tree = parser.parse(text, None).expect("could not parse text");
    let root = tree.root_node();

    let query = r#"
    (block_mapping_pair
        key: (flow_node) @key
        value: (block_node
            (block_sequence
                (block_sequence_item
                    (flow_node) @value)))
        (#eq? @key "mixins")
    )
    "#;

    let query = Query::new(&language, query).expect("could not create query");

    let mut cursor_qry = QueryCursor::new();

    let mut matches = cursor_qry.matches(&query, root, text.as_bytes());
    while let Some(m) = matches.next() {
        let found = m.captures.iter().any(|c| {
            if let Some(parent) = c.node.parent() {
                return parent.kind() == "block_mapping_pair"
                    && get_node_text(&c.node, text) == Some("mixins")
                    && is_cursor_within_node(&parent, pos);
            }
            false
        });
        if found {
            return true;
        }
    }
    false
}
