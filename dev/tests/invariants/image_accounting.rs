//! Spare capacity is prepaid once; rejected growth publishes no image or token.
//! A `crate` path root of derive text is the defining crate's root token.
use super::*;
use std::cell::Cell;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn unit() -> TestResult<crate::resolve::unit::Unit> {
    crate::resolve::unit::Unit::parse(
        crate::support::source::SourceFile::new(b"struct Image;".to_vec())?,
        &crate::expand::config::Config::default(),
        &mut crate::support::names::Names::default(),
    )
    .map_err(|error| error.message.into())
}
fn lengths(output: &Output<'_>) -> (usize, usize, usize) {
    (
        output.stream.source.len(),
        output.stream.origins.len(),
        output.stream.tokens.len(),
    )
}

#[test]
fn spare_capacity_and_failed_growth_keep_attempts_and_unpublished_rows() -> TestResult {
    use crate::{
        expand::{
            marks::{ExpansionId, SyntaxContextId},
            stream::{Location, Stream},
        },
        syntax::token_tree::Span,
    };
    let unit = unit()?;
    let site = super::super::Site {
        file: crate::support::source::FileId(0),
        source: unit.source.text(),
        ast: &unit.configured.ast,
        active: &unit.configured.active,
        item: unit.configured.ast.items[0],
        original: Span { start: 0, end: 1 },
        crate_id: 0,
        members: [None; super::super::MEMBERS.len()],
        const_: false,
        prior_copy: false,
        enum_default: None,
    };
    let usage = Cell::new((0_usize, 0_usize));
    let reject = Cell::new(false);
    let charge = |work: usize, bytes: usize| {
        let old = usage.get();
        usage.set((old.0 + work, old.1 + bytes));
        if reject.get() && bytes != 0 {
            Err(error(0, "rejected image payment"))
        } else {
            Ok(())
        }
    };
    let mut output = super::super::Output {
        site: &site,
        stream: Stream::default(),
        fields: Vec::new(),
        charge: &charge,
        context: (SyntaxContextId::ROOT, ExpansionId(0)),
        edition: crate::support::edition::Edition::Rust2024,
    };
    output.stream.source.reserve(128);
    output.stream.origins.reserve(8);
    output.stream.tokens.reserve(8);
    let anchor = Location {
        file: site.file,
        span: site.original,
    };
    let token = Token {
        kind: crate::lex::token::Kind::Identifier,
        start: 0,
        end: 3,
    };
    let before = usage.get();
    for _ in 0..4 {
        output.prepare_image(3).map_err(|error| error.message)?;
        output
            .stream
            .image("abc", anchor, None, ExpansionId(0))
            .map_err(|error| error.message)?;
        output.push_token(token).map_err(|error| error.message)?;
    }
    assert_eq!(usage.get().1, before.1);
    assert!(usage.get().0 > before.0);
    reject.set(true);
    let before_lengths = lengths(&output);
    let before = usage.get();
    assert!(
        output
            .prepare_image(output.stream.source.capacity() + 1)
            .is_err()
    );
    assert_eq!(lengths(&output), before_lengths);
    assert!(usage.get().0 > before.0 && usage.get().1 > before.1);
    output.stream.tokens.shrink_to_fit();
    assert!(output.push_token(token).is_err());
    assert_eq!(lengths(&output), before_lengths);
    assert!(output.prepare_field().is_err());
    assert!(output.fields.is_empty());
    assert!(output.prepare_parser().is_err());
    assert_eq!(lengths(&output), before_lengths);
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    Ok(())
}

#[test]
fn derive_text_roots_name_the_defining_crate_and_keep_other_words() -> TestResult {
    use crate::{
        expand::{
            marks::{ExpansionId, Marks, SyntaxContextId, Transparency},
            stream::Stream,
        },
        lex::token::Kind as TokenKind,
        syntax::token_tree::Span,
    };
    let unit = unit()?;
    let introduction = ExpansionId(1);
    let context = Marks::default().apply(
        SyntaxContextId::ROOT,
        introduction,
        Transparency::Opaque,
        SyntaxContextId::ROOT,
    )?;
    for crate_id in [0, 7] {
        let site = super::super::Site {
            file: crate::support::source::FileId(0),
            source: unit.source.text(),
            ast: &unit.configured.ast,
            active: &unit.configured.active,
            item: unit.configured.ast.items[0],
            original: Span { start: 0, end: 1 },
            crate_id,
            members: [None; super::super::MEMBERS.len()],
            const_: false,
            prior_copy: false,
            enum_default: None,
        };
        let charge = |_: usize, _: usize| -> Result<(), crate::lex::token::LexError> { Ok(()) };
        let mut output = super::super::Output {
            site: &site,
            stream: Stream::default(),
            fields: Vec::new(),
            charge: &charge,
            context: (context, introduction),
            edition: crate::support::edition::Edition::Rust2024,
        };
        output
            .text("crate::clone::Clone for crated r#type")
            .map_err(|error| error.message)?;
        let kinds: Vec<_> = output
            .stream
            .tokens
            .iter()
            .map(|token| token.kind)
            .collect();
        assert_eq!(
            kinds[0],
            TokenKind::crate_root(crate_id).map_err(|error| error.message)?
        );
        assert!(
            kinds[1..]
                .iter()
                .all(|kind| !matches!(kind, TokenKind::CrateRoot(_)))
        );
        assert_eq!(kinds.last(), Some(&TokenKind::RawIdentifier));
    }
    Ok(())
}
