//! Check the retained documentation grammar without changing target cfg.
use crate::{
    lex::token::{Kind, Punctuation},
    syntax::token_tree::{Delimiter, Forest, TreeKind, TreeRange},
};

pub(super) fn invalid(forest: &Forest, source: &str, start: u32, end: u32) -> Option<&'static str> {
    if end == start + 1 {
        return None;
    }
    if forest.punctuation(start + 1, Punctuation::Equal) {
        return if end == start + 3
            && matches!(forest.name(source, start + 2), Some("true" | "false"))
        {
            None
        } else {
            Some("doc(auto_cfg) requires a boolean value")
        };
    }
    let Some(children) = group(forest, start + 1, end) else {
        return Some("doc(auto_cfg) requires a boolean or a parenthesized list");
    };
    for (item, end) in entries(forest, children) {
        if !matches!(forest.name(source, item), Some("hide" | "show")) {
            return Some("doc(auto_cfg) accepts only hide and show");
        }
        let Some(items) = group(forest, item + 1, end) else {
            return Some("doc(auto_cfg) hide and show require a parenthesized list");
        };
        if let Some(message) = invalid_hide_show(forest, source, items) {
            return Some(message);
        }
    }
    None
}

fn invalid_hide_show(forest: &Forest, source: &str, items: TreeRange) -> Option<&'static str> {
    let mut has_name = false;
    let mut has_values = false;
    for (item, end) in entries(forest, items) {
        if has_values {
            return Some("doc(auto_cfg) values must be the last item");
        }
        let name = forest.name(source, item);
        if name.is_some() && end == item + 1 {
            has_name = true;
        } else if name == Some("values")
            && let Some(values) = group(forest, item + 1, end)
        {
            if !has_name {
                return Some("doc(auto_cfg) values require a preceding cfg name");
            }
            if let Some(message) = invalid_values(forest, source, values) {
                return Some(message);
            }
            has_values = true;
        } else {
            return Some("doc(auto_cfg) hide and show require cfg names or values");
        }
    }
    None
}

fn invalid_values(forest: &Forest, source: &str, items: TreeRange) -> Option<&'static str> {
    let mut strings = false;
    let mut any = false;
    for (item, end) in entries(forest, items) {
        if end == item + 1
            && forest
                .token(item)
                .is_some_and(|token| matches!(token.kind, Kind::String | Kind::RawString))
        {
            strings = true;
        } else if matches!(forest.name(source, item), Some("any" | "none"))
            && group(forest, item + 1, end).is_some_and(|range| range.start == range.end)
        {
            any |= forest.name(source, item) == Some("any");
        } else {
            return Some("doc(auto_cfg) values require strings, any(), or none()");
        }
        if any && strings {
            return Some("doc(auto_cfg) cannot mix any() with named values");
        }
    }
    None
}

fn group(forest: &Forest, start: u32, end: u32) -> Option<TreeRange> {
    if start >= end || forest.after(start) != end {
        return None;
    }
    match forest.tree(start)?.kind {
        TreeKind::Group {
            delimiter: Delimiter::Parenthesis,
            ..
        } => forest.children(start),
        _ => None,
    }
}

fn entries(forest: &Forest, range: TreeRange) -> impl Iterator<Item = (u32, u32)> {
    let mut cursor = range.start;
    std::iter::from_fn(move || {
        let start = cursor;
        if start == range.end {
            return None;
        }
        while cursor < range.end && !forest.punctuation(cursor, Punctuation::Comma) {
            cursor = forest.after(cursor);
        }
        let end = cursor;
        cursor += u32::from(cursor < range.end);
        Some((start, end))
    })
}
