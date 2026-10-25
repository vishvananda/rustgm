//! Typed metadata relocation records demanded token spans once per occurrence.
use super::*;
use crate::{
    lex::token::Token,
    syntax::{
        attribute::{Attribute, Input, Style},
        interpolation::{Nonterminal, Slice, Splice},
        token_tree::ForestBuilder,
    },
};
type Result<T = ()> = std::result::Result<T, crate::lex::token::LexError>;
fn opaque(source: &str, span: Span) -> Result<Forest> {
    let mut builder = ForestBuilder::default();
    builder.push(
        source,
        Token {
            kind: Kind::interpolated(0)?,
            start: span.start,
            end: span.end,
        },
    )?;
    builder.finish(span.end)
}
#[test]
fn captured_cfg_metadata_keeps_exact_once_relocated_uses_across_forwarding() -> Result {
    let original = "ub_checks";
    let trees = Forest::parse(original)?;
    let attribute = Attribute {
        span: Span { start: 0, end: 9 },
        style: Style::Outer,
        path: trees.roots(),
        input: Input::Empty,
        unsafe_wrapper: false,
    };
    let source = format!("{}ub_checks", " ".repeat(32));
    let destination = Span { start: 32, end: 41 };
    let splice = Splice {
        fragment: Nonterminal::Metadata {
            forest: &trees,
            attribute,
            expression: None,
        },
        original_source: original,
        original: attribute.span,
        destination,
    };
    let forest = opaque(&source, destination)?;
    let mut config = Config::default();
    for enabled in [false, true] {
        config.set_runtime_checks(
            crate::support::runtime_checks::RuntimeCheckConfig::with_ub_checks(enabled),
        );
        let values = [splice];
        let answer = evaluate(
            &forest,
            &source,
            forest.roots(),
            &config,
            &Slice(&values),
            0,
        )?;
        assert_eq!(answer.value, enabled);
        assert_eq!(answer.uses.len(), 1);
        assert_eq!(answer.uses[0].span, destination);
    }
    let relocated = splice.metadata()?;
    let forwarded_source = format!("{}ub_checks", " ".repeat(64));
    let forwarded_span = Span { start: 64, end: 73 };
    let forwarded = Splice {
        fragment: Nonterminal::Metadata {
            forest: &relocated.forest,
            attribute: relocated.attribute,
            expression: None,
        },
        original_source: &source,
        original: destination,
        destination: forwarded_span,
    };
    let forest = opaque(&forwarded_source, forwarded_span)?;
    let values = [forwarded];
    let answer = evaluate(
        &forest,
        &forwarded_source,
        forest.roots(),
        &config,
        &Slice(&values),
        0,
    )?;
    assert!(answer.value);
    assert_eq!(answer.uses.len(), 1);
    assert_eq!(answer.uses[0].span, forwarded_span);
    let invalid = Splice {
        original: Span { start: 33, end: 41 },
        ..forwarded
    };
    let values = [invalid];
    assert!(
        evaluate(
            &forest,
            &forwarded_source,
            forest.roots(),
            &config,
            &Slice(&values),
            0
        )
        .is_err()
    );
    Ok(())
}
