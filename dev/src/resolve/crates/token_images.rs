//! One preorder walk spells retained macro trees as the canonical metadata image.
//! Providers publish it; consumers re-lex the image once and must rebuild it exactly.
//!
//! A piece is separated from the previous piece exactly when its source span
//! leaves a gap. Trees come from the lexer, so every gap is whitespace or
//! comments, and every adjacent run of pieces is the exact source substring.
use crate::{
    lex::token::Kind,
    support::crates::exported_macros::{Delimiter as Wire, ImageBuilder, Row, TokenClass},
    syntax::token_tree::{Delimiter, Forest, TreeKind},
};

#[cfg(test)]
#[path = "../../../tests/invariants/joint_images.rs"]
mod tests;

/// # Errors
/// Rejects doc trees, generated tokens and budget overflow by name.
pub(crate) fn canonical(source: &str, forest: &Forest) -> Result<(String, Vec<Row>), &'static str> {
    let mut builder = ImageBuilder::default();
    let mut previous = None;
    // Each open group keeps its first following row and its close delimiter's start.
    let mut closes: Vec<(u32, u32)> = Vec::new();
    for (index, tree) in forest.nodes().iter().enumerate() {
        let index = u32::try_from(index).map_err(|_| "exported macro body exceeds its budget")?;
        while let Some(&(after, close)) = closes.last()
            && after == index
        {
            closes.pop();
            builder.close(separated(&mut previous, close, close + 1)?)?;
        }
        match tree.kind {
            TreeKind::Token(kind) => {
                let spelling = source
                    .get(tree.span.start as usize..tree.span.end as usize)
                    .ok_or("exported macro token lies outside its source")?;
                let gap = separated(&mut previous, tree.span.start, tree.span.end)?;
                builder.token(class(kind)?, spelling, gap)?;
            }
            TreeKind::Group { delimiter, after } => {
                // The open delimiter is the group's first byte, the close its last.
                let close = tree
                    .span
                    .end
                    .checked_sub(1)
                    .filter(|&close| close > tree.span.start)
                    .ok_or("exported macro group span is malformed")?;
                let gap = separated(&mut previous, tree.span.start, tree.span.start + 1)?;
                builder.open(
                    match delimiter {
                        Delimiter::Parenthesis => Wire::Parenthesis,
                        Delimiter::Bracket => Wire::Bracket,
                        Delimiter::Brace => Wire::Brace,
                    },
                    gap,
                )?;
                closes.push((after, close));
            }
            TreeKind::Doc { .. } => {
                return Err(
                    "doc comments in exported macro bodies are outside the macro metadata profile",
                );
            }
        }
    }
    while let Some((_, close)) = closes.pop() {
        builder.close(separated(&mut previous, close, close + 1)?)?;
    }
    builder.finish()
}

/// Whether the source left a gap before the piece at `start..end`, which then
/// becomes the previous piece. Pieces of one forest never overlap.
fn separated(previous: &mut Option<u32>, start: u32, end: u32) -> Result<bool, &'static str> {
    let gap = match *previous {
        None => false,
        Some(before) if before <= start => before < start,
        Some(_) => return Err("exported macro token trees overlap"),
    };
    *previous = Some(end);
    Ok(gap)
}

fn class(kind: Kind) -> Result<TokenClass, &'static str> {
    Ok(match kind {
        Kind::Identifier => TokenClass::Identifier,
        Kind::RawIdentifier => TokenClass::RawIdentifier,
        Kind::Lifetime => TokenClass::Lifetime,
        Kind::RawLifetime => TokenClass::RawLifetime,
        Kind::Integer => TokenClass::Integer,
        Kind::Float => TokenClass::Float,
        Kind::Character => TokenClass::Character,
        Kind::Byte => TokenClass::Byte,
        Kind::String => TokenClass::String,
        Kind::ByteString => TokenClass::ByteString,
        Kind::CString => TokenClass::CString,
        Kind::RawString => TokenClass::RawString,
        Kind::RawByteString => TokenClass::RawByteString,
        Kind::RawCString => TokenClass::RawCString,
        Kind::Punctuation(_) => TokenClass::Punctuation,
        Kind::Interpolated(_)
        | Kind::CrateRoot(_)
        | Kind::UnknownPrefix
        | Kind::Whitespace
        | Kind::LineComment
        | Kind::BlockComment
        | Kind::OuterLineDoc
        | Kind::InnerLineDoc
        | Kind::OuterBlockDoc
        | Kind::InnerBlockDoc => {
            return Err("exported macro token is outside the macro metadata profile");
        }
    })
}
