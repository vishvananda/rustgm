//! One preorder walk spells retained macro trees as the canonical metadata image.
//! Providers publish it; consumers re-lex the image once and must rebuild it exactly.
use crate::{
    lex::token::Kind,
    support::crates::exported_macros::{Delimiter as Wire, ImageBuilder, Row, TokenClass},
    syntax::token_tree::{Delimiter, Forest, TreeKind},
};

/// # Errors
/// Rejects doc trees, generated tokens and budget overflow by name.
pub(crate) fn canonical(source: &str, forest: &Forest) -> Result<(String, Vec<Row>), &'static str> {
    let mut builder = ImageBuilder::default();
    let mut closes: Vec<u32> = Vec::new();
    for (index, tree) in forest.nodes().iter().enumerate() {
        let index = u32::try_from(index).map_err(|_| "exported macro body exceeds its budget")?;
        while closes.last() == Some(&index) {
            closes.pop();
            builder.close()?;
        }
        match tree.kind {
            TreeKind::Token(kind) => {
                let spelling = source
                    .get(tree.span.start as usize..tree.span.end as usize)
                    .ok_or("exported macro token lies outside its source")?;
                builder.token(class(kind)?, spelling)?;
            }
            TreeKind::Group { delimiter, after } => {
                builder.open(match delimiter {
                    Delimiter::Parenthesis => Wire::Parenthesis,
                    Delimiter::Bracket => Wire::Bracket,
                    Delimiter::Brace => Wire::Brace,
                })?;
                closes.push(after);
            }
            TreeKind::Doc { .. } => {
                return Err(
                    "doc comments in exported macro bodies are outside the macro metadata profile",
                );
            }
        }
    }
    while closes.pop().is_some() {
        builder.close()?;
    }
    builder.finish()
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
