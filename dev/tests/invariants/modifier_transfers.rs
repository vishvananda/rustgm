//! The sparse rows follow typed copies, rollback debt and genuine original replay.
use super::*;
use crate::syntax::{
    expression::{
        self,
        fragment::{self, FragmentKind},
    },
    token_tree::Span,
    transfer::{self, Extent},
};
fn rollback_and_retry() -> TestResult {
    let text = "struct S;impl S{default fn f(){} default fn g(){}}";
    let source = expression::parse_crate(text).map_err(|error| error.message)?;
    let rows = source.gate_sites.as_ref().ok_or("source default rows")?;
    assert_eq!(rows.len(), 2);
    let rejected = rows.entries()[1].span;
    let mut destination = Syntax::default();
    assert!(destination.gate_sites.is_none());
    let empty = Extent::new(&destination);
    let mut previous = 0;
    for _ in 0..3 {
        let result = transfer::append(
            &source,
            &mut destination,
            |span| {
                if span == rejected {
                    Err(crate::syntax::token_tree::error(
                        span.start,
                        "declared transfer failure",
                    ))
                } else {
                    Ok(span)
                }
            },
            |value| Ok(value.clone()),
        );
        assert!(result.is_err());
        assert!(destination.nodes.is_empty() && destination.items.is_empty());
        let retained = destination
            .gate_sites
            .as_ref()
            .ok_or("zero-prefix debt owner lost")?;
        let (len, capacity, steps, bytes) = retained.storage();
        assert_eq!(len, 0);
        assert!(capacity > 0 && steps > previous && bytes > 0);
        previous = steps;
        let copied = empty.copy(&destination).map_err(|error| error.message)?;
        assert!(
            copied
                .gate_sites
                .as_ref()
                .ok_or("replay discarded failed work")?
                .storage()
                .2
                >= steps
        );
    }
    let root = transfer::append(&source, &mut destination, Ok, |value| Ok(value.clone()))
        .map_err(|error| error.message)?;
    destination.root = root;
    let copied = destination
        .gate_sites
        .as_ref()
        .ok_or("successful retry rows")?;
    assert_eq!(copied.len(), 2);
    assert!(copied.storage().2 > previous);
    for row in copied.entries() {
        row.validate(&destination).map_err(|error| error.message)?;
    }
    Ok(())
}
fn tokens(text: &str) -> TestResult<Vec<crate::lex::token::Token>> {
    let mut lexer = crate::lex::Lexer::new(text).map_err(|error| error.message)?;
    let mut tokens = Vec::new();
    while let Some(token) = lexer.next_token().map_err(|error| error.message)? {
        if !matches!(
            token.kind,
            crate::lex::token::Kind::Whitespace
                | crate::lex::token::Kind::LineComment
                | crate::lex::token::Kind::BlockComment
        ) {
            tokens.push(token);
        }
    }
    Ok(tokens)
}
fn root_replacement_and_originals() -> TestResult {
    let original = "#![feature(min_specialization)]struct S;impl S{m!();}fn main(){}";
    let mut names = crate::support::names::Names::default();
    let mut unit = Unit::parse(
        SourceFile::new(original.as_bytes().to_vec())?,
        &Config::default(),
        &mut names,
    )
    .map_err(|error| error.message)?;
    let target = unit
        .configured
        .ast
        .nodes
        .iter()
        .position(|node| matches!(node.kind, crate::syntax::ast::Kind::Macro { .. }))
        .ok_or("actual pending macro")?;
    let target = NodeId::new(u32::try_from(target)?);
    let prior = unit.configured.ast.node(target);
    let text = "default fn f()->u8{1}";
    let tokens = tokens(text)?;
    let parsed = fragment::parse(text, &tokens, FragmentKind::Item)
        .map_err(|error| error.message)?
        .ast;
    assert!(matches!(
        parsed.node(parsed.root).kind,
        crate::syntax::ast::Kind::Item(_)
    ));
    let generated = crate::expand::crate_syntax::configure(text, parsed, &Config::default())
        .map_err(|error| error.message)?;
    let origin = crate::expand::stream::Origin {
        generated: Span {
            start: 0,
            end: u32::try_from(text.len())?,
        },
        original: crate::expand::stream::Location {
            file: FileId(0),
            span: prior.span,
        },
        context: Some(crate::expand::marks::SyntaxContextId::ROOT),
        introduction: crate::expand::marks::ExpansionId(0),
        joint: None,
    };
    unit.install_expansion(target, text, generated, &[origin])
        .map_err(|error| error.message)?;
    let row = unit
        .configured
        .ast
        .gate_sites
        .as_ref()
        .ok_or("committed root row")?
        .entries()[0];
    assert_eq!(row.node, target);
    assert!(unit.configured.active[target.get() as usize]);
    row.validate(&unit.configured.ast)
        .map_err(|error| error.message)?;
    assert!(
        row.context_key().is_none(),
        "a lookup must not seal an unvisited output"
    );
    unit.configured
        .ast
        .validate_gate_sites_source(unit.source.original_image())
        .map_err(|error| error.message)?;
    for _ in 0..2 {
        let fresh = unit.fresh_instance().map_err(|error| error.message)?;
        assert_eq!(fresh.configured.ast.node(target).kind, prior.kind);
        let debt = fresh
            .configured
            .ast
            .gate_sites
            .as_ref()
            .ok_or("fresh replay lost retention debt")?;
        assert_eq!(debt.len(), 0);
        assert!(debt.storage().2 > 0);
        assert_eq!(
            fresh.source.original().as_ptr(),
            unit.source.original().as_ptr()
        );
        assert!(!std::sync::Arc::ptr_eq(
            fresh.source.original_image(),
            unit.source.original_image()
        ));
        fresh
            .configured
            .ast
            .validate_gate_sites_source(fresh.source.original_image())
            .map_err(|error| error.message)?;
    }
    Ok(())
}
#[test]
fn sparse_modifiers_preserve_transfer_retry_root_and_original_identities() -> TestResult {
    rollback_and_retry()?;
    root_replacement_and_originals()?;
    for (lane, name, expected) in [
        ("oracle-v1", "single-default-method", DefaultKind::Function),
        (
            "oracle-v1",
            "partial-impl-child-inherits",
            DefaultKind::Implementation,
        ),
        ("oracle-v1", "full-default-constant", DefaultKind::Constant),
        (
            "soft-edges",
            "full-default-associated-type",
            DefaultKind::Type,
        ),
    ] {
        let text = fixture(lane, name)?;
        let ast = expression::parse_crate(&text).map_err(|error| error.message)?;
        let rows = ast.gate_sites.as_ref().ok_or("parsed modifier rows")?;
        assert_eq!(rows.len(), 1, "{name}");
        assert_eq!(rows.entries()[0].kind, expected);
        let mut output = Vec::new();
        expression::render::write_ast(&text, &ast, &mut output)?;
        assert!(String::from_utf8(output)?.contains(" default "));
    }
    let plain = expression::parse_crate(
        "fn default(){}macro_rules! default{()=>{}}fn main(){default();default!();}",
    )
    .map_err(|error| error.message)?;
    assert!(plain.gate_sites.is_none());
    Ok(())
}
