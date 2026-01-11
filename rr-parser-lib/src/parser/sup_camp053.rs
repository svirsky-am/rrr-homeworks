use crate::parser::errors::ParseError;

pub fn get_text(node: roxmltree::Node, tag: (&str, &str)) -> String {
    node.children()
        .find(|n| n.has_tag_name(tag))
        .and_then(|n| n.text())
        .unwrap_or("")
        .trim()
        .to_string()
}

pub fn find_nested_text(parent: roxmltree::Node, path: &[(&str, &str)]) -> String {
    let mut current = parent;
    for &tag in path {
        current = match current.children().find(|n| n.has_tag_name(tag)) {
            Some(n) => n,
            None => return String::new(),
        };
    }
    current.text().unwrap_or("").trim().to_string()
}

pub fn get_text_of_deep_child_node<'a>(
    parent: roxmltree::Node<'a, 'a>,
    tag: &'a str,
) -> Option<&'a str> {
    if let Some(accptnc_dt_tm_str) = parent.descendants().find(|n| n.tag_name().name() == tag) {
        if let Some(text) = accptnc_dt_tm_str.text() {
            Some(text)
        } else {
            None
        }
    } else {
        None
    }
}

pub fn get_text_or_error<'a>(
    node: roxmltree::Node,
    tag: (&'a str, &'a str),
) -> Result<String, ParseError> {
    let text = get_text(node, tag);
    if text.is_empty() {
        Err(ParseError::Camt053MissingTextContent)
    } else {
        Ok(text)
    }
}
