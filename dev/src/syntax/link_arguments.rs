//! Native-link arguments retain tree identities; cfg evaluation belongs to expansion.
use super::{
    attribute::{Attribute, Input},
    token_tree::{Delimiter, Forest, TreeKind, TreeRange, error},
};
use crate::lex::token::{LexError, Punctuation};

pub(crate) struct Arguments<'a> {
    forest: &'a Forest,
    remaining: TreeRange,
}
pub(crate) fn arguments(forest: &Forest, attribute: Attribute) -> Result<Arguments<'_>, LexError> {
    if attribute.unsafe_wrapper {
        return Err(error(
            attribute.span.start,
            "link is not an unsafe attribute",
        ));
    }
    let Input::Delimited(group) = attribute.input else {
        return Err(error(
            attribute.span.start,
            "link requires parenthesized arguments",
        ));
    };
    let remaining = parenthesized(forest, group)?;
    Ok(Arguments { forest, remaining })
}
impl Iterator for Arguments<'_> {
    type Item = TreeRange;
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining.start == self.remaining.end {
            return None;
        }
        let start = self.remaining.start;
        let mut end = start;
        while end < self.remaining.end && !self.forest.punctuation(end, Punctuation::Comma) {
            end = self.forest.after(end);
        }
        self.remaining.start = end + u32::from(end < self.remaining.end);
        Some(TreeRange { start, end })
    }
}
pub(crate) fn predicate(forest: &Forest, range: TreeRange) -> Result<TreeRange, LexError> {
    let group = range.start + 1;
    if group >= range.end || forest.after(group) != range.end {
        return Err(error(
            forest.offset(range.start),
            "link cfg requires one predicate list",
        ));
    }
    parenthesized(forest, group)
}
fn parenthesized(forest: &Forest, group: u32) -> Result<TreeRange, LexError> {
    if !matches!(
        forest.tree(group).map(|tree| tree.kind),
        Some(TreeKind::Group {
            delimiter: Delimiter::Parenthesis,
            ..
        })
    ) {
        return Err(error(
            forest.offset(group),
            "expected parenthesized link arguments",
        ));
    }
    forest
        .children(group)
        .ok_or_else(|| error(forest.offset(group), "missing link argument list"))
}
