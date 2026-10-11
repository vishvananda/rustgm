//! Attribute syntax retains typed tree ranges until the owning expansion phase.
use super::token_tree::{Delimiter, Forest, Span, TreeKind, TreeRange, error};
use crate::lex::{
    keyword,
    token::{Kind, LexError, Punctuation},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Input {
    Empty,
    Delimited(u32),
    Equals(TreeRange),
    Doc(Span),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Style {
    Outer,
    Inner,
}

impl Style {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Outer => "outer",
            Self::Inner => "inner",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Attribute {
    pub span: Span,
    pub style: Style,
    pub path: TreeRange,
    pub input: Input,
    pub unsafe_wrapper: bool,
}

impl Attribute {
    #[must_use]
    pub fn name<'source>(&self, forest: &Forest, source: &'source str) -> Option<&'source str> {
        if matches!(self.input, Input::Doc(_)) {
            return Some("doc");
        }
        if self.path.end != self.path.start + 1 {
            return None;
        }
        forest.name(source, self.path.start)
    }
}

/// Parse an attribute at a syntax-owned position. Does not search macro input.
/// The returned index is the next sibling after the complete attribute.
///
/// # Errors
/// Reports malformed # attributes, paths, unsafe wrappers or empty RHS input.
pub fn at(forest: &Forest, source: &str, index: u32) -> Result<Option<(Attribute, u32)>, LexError> {
    let Some(tree) = forest.tree(index) else {
        return Ok(None);
    };
    if let TreeKind::Doc { inner, body } = tree.kind {
        return Ok(Some((
            Attribute {
                span: tree.span,
                style: if inner { Style::Inner } else { Style::Outer },
                path: TreeRange {
                    start: index,
                    end: index,
                },
                input: Input::Doc(body),
                unsafe_wrapper: false,
            },
            index + 1,
        )));
    }
    if !forest.punctuation(index, Punctuation::Pound) {
        return Ok(None);
    }
    let mut bracket = index + 1;
    let style = if forest.punctuation(bracket, Punctuation::Bang) {
        bracket += 1;
        Style::Inner
    } else {
        Style::Outer
    };
    let Some(group) = forest.tree(bracket) else {
        return Err(error(tree.span.start, "expected attribute bracket"));
    };
    if !matches!(
        group.kind,
        TreeKind::Group {
            delimiter: Delimiter::Bracket,
            ..
        }
    ) {
        return Err(error(group.span.start, "expected attribute bracket"));
    }
    let range = forest
        .children(bracket)
        .ok_or_else(|| error(group.span.start, "expected attribute body"))?;
    let span = Span {
        start: tree.span.start,
        end: group.span.end,
    };
    Ok(Some((
        parse(forest, source, range, style, span)?,
        forest.after(bracket),
    )))
}

/// Parse the Attr production, also used for `cfg_attr`'s nested attributes.
///
/// # Errors
/// Reports malformed paths, wrappers or attribute input.
pub fn parse(
    forest: &Forest,
    source: &str,
    mut range: TreeRange,
    style: Style,
    span: Span,
) -> Result<Attribute, LexError> {
    let unsafe_wrapper = forest.name(source, range.start) == Some("unsafe")
        && forest
            .token(range.start)
            .is_some_and(|token| token.kind == Kind::Identifier);
    if unsafe_wrapper {
        let group = range.start + 1;
        if forest.after(group) != range.end
            || !matches!(
                forest.tree(group).map(|t| t.kind),
                Some(TreeKind::Group {
                    delimiter: Delimiter::Parenthesis,
                    ..
                })
            )
        {
            return Err(error(
                span.start,
                "unsafe attribute requires one parenthesized attribute",
            ));
        }
        range = forest
            .children(group)
            .ok_or_else(|| error(span.start, "missing unsafe attribute"))?;
        if forest.name(source, range.start) == Some("unsafe")
            && forest
                .token(range.start)
                .is_some_and(|token| token.kind == Kind::Identifier)
        {
            return Err(error(span.start, "nested unsafe attribute wrapper"));
        }
    }
    let mut next = range.start;
    if forest.punctuation(next, Punctuation::ColonColon) {
        next += 1;
    }
    loop {
        if next >= range.end || !path_segment(forest, source, next) {
            return Err(error(
                forest.offset(next),
                "expected attribute path segment",
            ));
        }
        next += 1;
        if next >= range.end || !forest.punctuation(next, Punctuation::ColonColon) {
            break;
        }
        next += 1;
    }
    let path = TreeRange {
        start: range.start,
        end: next,
    };
    let input = if next == range.end {
        Input::Empty
    } else if forest.punctuation(next, Punctuation::Equal) {
        if next + 1 == range.end {
            return Err(error(forest.offset(next), "expected attribute value"));
        }
        Input::Equals(TreeRange {
            start: next + 1,
            end: range.end,
        })
    } else if forest.children(next).is_some() && forest.after(next) == range.end {
        Input::Delimited(next)
    } else {
        return Err(error(forest.offset(next), "expected one attribute input"));
    };
    Ok(Attribute {
        span,
        style,
        path,
        input,
        unsafe_wrapper,
    })
}

fn path_segment(forest: &Forest, source: &str, index: u32) -> bool {
    let Some(token) = forest.token(index) else {
        return false;
    };
    let Some(name) = forest.name(source, index) else {
        return false;
    };
    if token.kind == Kind::RawIdentifier {
        !matches!(name, "_" | "self" | "Self" | "crate" | "super")
    } else {
        !keyword::reserved(name) || matches!(name, "self" | "crate" | "super")
    }
}
