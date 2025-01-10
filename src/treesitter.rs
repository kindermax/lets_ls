use streaming_iterator::StreamingIterator;
use tree_sitter::{Node, Parser, Query, QueryCursor};


#[derive(Debug)]
pub enum PositionType {
    Mixins,
    Depends,
    None,
}

fn is_cursor_within_node(node: &Node, pos: &lsp_types::Position) -> bool {
    let start_point = node.start_position();
    let end_point = node.end_position();

    pos.line as usize >= start_point.row
        && pos.line as usize <= end_point.row
        && pos.character as usize >= start_point.column
        && pos.character as usize <= end_point.column
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

pub fn word_before_cursor(
    line: &str,
    char_index: usize,
    predicate: fn(c: char) -> bool,
) -> &str {
    if char_index == 0 || char_index > line.len() {
        return "";
    }

    let start = line[..char_index]
        .rfind(predicate)
        .map_or(0, |index| index + 1);

    if start == char_index {
        return "";
    }

    &line[start..char_index]
}

pub fn word_after_cursor(
    line: &str,
    char_index: usize,
    predicate: fn(c: char) -> bool,
) -> &str {
    if char_index >= line.len() {
        return "";
    }

    let start = char_index;

    let end = line[start..]
        .char_indices()
        .find(|&(_, c)| predicate(c))
        .map_or(line.len(), |(idx, _)| start + idx);

    &line[start..end]
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
                return parent.kind() == "block_sequence_item" && is_cursor_at_line(&c.node, pos);
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

pub fn is_depends_node(text: &str, pos: &lsp_types::Position) -> bool {
    let mut parser = Parser::new();
    let language = tree_sitter_yaml::LANGUAGE.into();
    parser
        .set_language(&language)
        .expect("could not load yaml language");

    let tree = parser.parse(text, None).expect("could not parse text");
    let root = tree.root_node();

    let query = r#"
    (
        block_mapping_pair
        key: (flow_node)@keydepends
        value: [
            (flow_node(flow_sequence(flow_node(plain_scalar(string_scalar))))) @depends
            (block_node(block_sequence(block_sequence_item) @depends))
        ]
        (#eq? @keydepends "depends")
    )
    "#;

    let query = Query::new(&language, query).expect("could not create query");

    let mut cursor_qry = QueryCursor::new();

    let depends_idx = query.capture_index_for_name("depends").unwrap();

    let mut matches = cursor_qry.matches(&query, root, text.as_bytes());
    while let Some(m) = matches.next() {
        let found = m.captures.iter().any(|c| {
                if c.index == depends_idx {
                    let kind = c.node.kind();
                    let text = get_node_text(&c.node, text).unwrap();
                    eprintln!("kind: {}, node: {:?}, pos: {:?}, text: {}", kind, c.node, pos, text);
                    match kind {
                        "block_sequence_item" => {
                            return is_cursor_within_node(&c.node, pos) || is_cursor_at_line(&c.node, pos);
                        },
                        "flow_sequence" | "flow_node" => {
                            return is_cursor_within_node(&c.node, pos);
                        },
                        _ => return false,
                    }
                }
                return false;
        });
        if found {
            return true;
        }
    }
    false
}


pub fn get_position_type(doc: &str, pos: lsp_types::Position) -> PositionType {
    if is_mixin_root_node(doc, &pos) {
        return PositionType::Mixins;
    } else if is_depends_node(doc, &pos) {
        return PositionType::Depends;
    }
    PositionType::None
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Command {
    pub name: String,
}

pub fn get_commands(doc: &str) -> Vec<Command> {
    let mut parser = Parser::new();
    let language = tree_sitter_yaml::LANGUAGE.into();
    parser
        .set_language(&language)
        .expect("could not load yaml language");

    let tree = parser.parse(doc, None).expect("could not parse text");
    let root = tree.root_node();

    let query = r#"
    (
        stream(
            document(
            block_node(
                block_mapping(
                block_mapping_pair
                    key: (flow_node(plain_scalar(string_scalar)@parent))
                    value: (block_node
                            (block_mapping
                                (block_mapping_pair
                                    key: (flow_node
                                        (plain_scalar
                                            (string_scalar)@cmd_key))
                                    value: (block_node)@cmd)@values))
                )
            )
            )
        )
        (#eq? @parent "commands")
    )
    "#;


    let query = Query::new(&language, query).expect("could not create query");

    let mut cursor_qry = QueryCursor::new();

    let mut matches = cursor_qry.matches(&query, root, doc.as_bytes());
    let mut commands = vec![];

    while let Some(m) = matches.next() {
        let mut command = Command {
            ..Command::default()
        };
        for c in m.captures {
            if c.index == 1 { // key, string_scalar
                command.name = get_node_text(&c.node, doc).unwrap().to_string();
            }
        }
        commands.push(command);
    }

    commands
}

#[cfg(test)]
mod tests {
    use super::*;
    use lsp_types::Position;

    #[test]
    fn test_detect_mixins_node() {
        let doc = r#"
shell: bash
mixins:
  - lets.my.yaml
commands:
  test:
    cmd: echo Test"#
            .trim();

        let tests = vec![
            (Position::new(0, 0), false),
            (Position::new(1, 0), true),
            (Position::new(2, 0), true),
            (Position::new(2, 15), true),
            (Position::new(3, 0), false),
        ];
        for (i, (pos, expect)) in tests.into_iter().enumerate() {
            let result = is_mixin_root_node(doc, &pos);
            assert_eq!(
                result, expect,
                "Case {i}: expected {expect}, actual {result}"
            );
        }
    }

    #[test]
    fn test_detect_depends_node_flow_node() {
        let doc = r#"
shell: bash
mixins:
  - lets.my.yaml
commands:
  test:
    cmd: echo Test

  test2:
    depends: [test]
    cmd: echo Test2"#
            .trim();

        let tests = vec![
            (Position::new(8, 15), true),
        ];
        for (i, (pos, expect)) in tests.into_iter().enumerate() {
            let result = is_depends_node(doc, &pos);
            assert_eq!(
                result, expect,
                "Case {i}: expected {expect}, actual {result}"
            );
        }
    }

    #[test]
    fn test_detect_depends_node_block_sequence_item() {
        let doc = r#"
shell: bash
mixins:
  - lets.my.yaml
commands:
  test:
    cmd: echo Test

  test2:
    depends:
      -
    cmd: echo Test2"#
            .trim();

        let tests = vec![
            (Position::new(8, 4), false),
            (Position::new(9, 0), true),
            (Position::new(9, 7), true),
            (Position::new(9, 8), true),
        ];
        for (i, (pos, expect)) in tests.into_iter().enumerate() {
            let result = is_depends_node(doc, &pos);
            assert_eq!(
                result, expect,
                "Case {i}: expected {expect}, actual {result}"
            );
        }
    }


    #[test]
    fn test_extract_filename_from_mixins_item() {
        let doc = r#"
shell: bash
mixins:
  - lets.my.yaml
commands:
  test:
    cmd: echo Test"#
            .trim();

        let tests = vec![
            (Position::new(1, 0), None),
            (Position::new(2, 0), Some("lets.my.yaml".to_string())),
            (Position::new(2, 15), Some("lets.my.yaml".to_string())),
        ];
        for (i, (pos, expect)) in tests.into_iter().enumerate() {
            let result = extract_filename(doc, &pos);
            assert_eq!(
                result, expect,
                "Case {i}: expected {expect:?}, actual {result:?}"
            );
        }
    }

    #[test]
    fn test_get_commands() {
        let doc = r#"
shell: bash
mixins:
  - lets.my.yaml
commands:
  test:
    cmd: echo Test
  test2:
    cmd: echo Test2"#
            .trim();

        let commands = get_commands(doc);
        assert_eq!(commands.len(), 2);
        assert_eq!(commands, vec![
            Command {
                name: "test".to_string(),
            },
            Command {
                name: "test2".to_string(),
            },
        ]);
    }
}
