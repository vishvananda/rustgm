//! Assert's private transaction is not permission to execute a source builtin.
use self::consume_test_support::*;
use super::*;
use crate::{
    expand::{config::Config, crate_syntax, marks::SyntaxContextId},
    lex::Lexer,
    support::source::FileId,
    syntax::{ast::Kind, interpolation::EMPTY_INPUTS},
};
use std::cell::Cell;
type Result = std::result::Result<(), LexError>;

#[test]
fn condition_parse_transfer_is_once_and_retains_real_suffix_and_spans() -> Result {
    for source in [
        "true",
        "true,",
        "{ false }",
        "1 + 2 == 4",
        "call(), \"{}\", message()",
    ] {
        let input = input(source)?;
        let prepared = prepare(
            Source {
                file: FileId(3),
                text: source,
            },
            &input,
            &EMPTY_INPUTS,
            &|_, _| Ok(()),
        )?;
        let count = prepared.ast.nodes.len();
        let high_water = prepared.ast.lookahead_high_water;
        let suffix = prepared.message.clone();
        let result = prepared.consume(
            "prefix ",
            |_| Ok((SyntaxContextId::ROOT, false)),
            &|_, _| Ok(()),
        )?;
        assert_eq!(result.parser_calls, 1);
        assert_eq!(result.ast.nodes.len(), count);
        assert_eq!(result.ast.lookahead_high_water, high_water);
        assert_eq!(result.message, suffix);
        assert!(result.stream.source.starts_with("prefix "));
        assert!(result.ast.nodes.iter().all(|node| node.span.start >= 7));
        assert!(
            result
                .stream
                .origins
                .iter()
                .all(|origin| origin.original.file == FileId(3))
        );
        assert_eq!(
            result.stream.tokens.len(),
            input.tokens.len()
                - suffix.len()
                - usize::from(
                    suffix.start > 0
                        && input
                            .tokens
                            .get(suffix.start - 1)
                            .is_some_and(|t| t.kind == TokenKind::Punctuation(Punctuation::Comma))
                )
        );
        let configured =
            crate_syntax::configure(&result.stream.source, result.ast, &Config::default())?;
        assert!(configured.active.iter().all(|enabled| *enabled));
    }
    for source in ["", "true;", "true \"message\"", "true false", "true, ("] {
        let input = input(source)?;
        if source == "true, (" {
            // Only the panic macro parses malformed message tokens at expansion.
            assert!(
                prepare(
                    Source {
                        file: FileId(0),
                        text: source
                    },
                    &input,
                    &EMPTY_INPUTS,
                    &|_, _| Ok(())
                )
                .is_ok()
            );
        } else {
            assert!(
                prepare(
                    Source {
                        file: FileId(0),
                        text: source
                    },
                    &input,
                    &EMPTY_INPUTS,
                    &|_, _| Ok(())
                )
                .is_err(),
                "{source}"
            );
        }
    }
    opaque::relocated_and_forwarded()?;
    assert!(crate::expand::builtin::Builtin::Assert.executable());
    Ok(())
}

#[test]
fn condition_attempt_and_transfer_budgets_fail_before_publication() -> Result {
    let source = "{ let value = 2u8; value == 2u8 }";
    let input = input(source)?;
    let steps = Cell::new(0usize);
    let bytes = Cell::new(0usize);
    let paid = |work, storage| {
        steps.set(steps.get() + work);
        bytes.set(bytes.get() + storage);
        Ok(())
    };
    let prepared = prepare(
        Source {
            file: FileId(0),
            text: source,
        },
        &input,
        &EMPTY_INPUTS,
        &paid,
    )?;
    let before = (steps.get(), bytes.get());
    assert!(
        prepared
            .consume("", |_| Ok((SyntaxContextId::ROOT, false)), &|work, _| {
                steps.set(steps.get() + work);
                Err(error(0, "test exhausted transfer budget"))
            })
            .is_err()
    );
    assert!(steps.get() > before.0);
    assert_eq!(bytes.get(), before.1);
    assert!(
        prepare(
            Source {
                file: FileId(0),
                text: source
            },
            &input,
            &EMPTY_INPUTS,
            &|_, _| Err(error(0, "test exhausted parse budget"))
        )
        .is_err()
    );
    let prepared = prepare(
        Source {
            file: FileId(0),
            text: source,
        },
        &input,
        &EMPTY_INPUTS,
        &paid,
    )?;
    assert!(
        prepared
            .consume("", |_| Err(error(0, "test changed origin")), &paid)
            .is_err()
    );
    let prepared = prepare(
        Source {
            file: FileId(0),
            text: source,
        },
        &input,
        &EMPTY_INPUTS,
        &paid,
    )?;
    let result = prepared.consume("", |_| Ok((SyntaxContextId::ROOT, false)), &paid)?;
    assert!(matches!(
        result.ast.node(result.ast.root).kind,
        Kind::Block { .. }
    ));
    Ok(())
}

#[path = "assert_emission.rs"]
mod emission;

mod consume_test_support {
    use super::*;
    pub(super) fn input(source: &str) -> std::result::Result<Input, crate::lex::token::LexError> {
        let mut lexer = Lexer::new(source)?;
        let mut tokens = Vec::new();
        while let Some(token) = lexer.next_token()? {
            if !matches!(
                token.kind,
                TokenKind::Whitespace | TokenKind::LineComment | TokenKind::BlockComment
            ) {
                tokens.push(token);
            }
        }
        let after = (1..=tokens.len()).collect();
        Ok(Input { tokens, after })
    }
}

#[path = "assert_fragments.rs"]
mod opaque;
