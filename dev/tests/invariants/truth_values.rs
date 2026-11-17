//! Boolean capture reads keep actual typed roots, spans and allocation behavior.
use super::*;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn checked<T>(result: Result<T, LexError>) -> TestResult<T> {
    result.map_err(|error| format!("{error:?}").into())
}
fn occurrence<'a>(source: &'a str, parsed: &'a ParsedSyntax) -> Splice<'a> {
    let original = parsed.ast.node(parsed.ast.root).span;
    Splice {
        fragment: Nonterminal::Syntax {
            kind: FragmentKind::Literal,
            parsed,
        },
        original_source: source,
        original,
        destination: Span {
            start: 48,
            end: 48 + original.end,
        },
    }
}
#[test]
fn stored_and_deferred_booleans_reuse_only_their_actual_literal_root() -> TestResult {
    for (source, expected) in [("true", true), ("false", false)] {
        let mut parsed = checked(ParsedSyntax::own(
            source,
            checked(super::super::expression::parse(source))?,
        ))?;
        let bytes = parsed.literals.decoded_bytes();
        for _ in 0..1000 {
            assert_eq!(
                checked(occurrence(source, &parsed).boolean_literal())?,
                expected
            );
        }
        assert_eq!(parsed.literals.decoded_bytes(), bytes);
        let token = checked(crate::lex::Lexer::new(source))?
            .next_token()
            .map_err(|e| format!("{e:?}"))?
            .ok_or("actual Boolean token")?;
        parsed.ast.nodes[parsed.ast.root.get() as usize].kind = Kind::DeferredLiteral(token);
        assert_eq!(
            checked(occurrence(source, &parsed).boolean_literal())?,
            expected
        );
    }
    Ok(())
}
#[test]
fn boolean_capture_rejects_foreign_spans_other_kinds_and_expression_roots() -> TestResult {
    let parsed = checked(ParsedSyntax::own(
        "true",
        checked(super::super::expression::parse("true"))?,
    ))?;
    let splice = occurrence("true", &parsed);
    assert!(
        Splice {
            original: Span { start: 1, end: 4 },
            ..splice
        }
        .boolean_literal()
        .is_err()
    );
    assert!(
        Splice {
            destination: Span { start: 48, end: 51 },
            ..splice
        }
        .boolean_literal()
        .is_err()
    );
    assert!(
        Splice {
            fragment: Nonterminal::Syntax {
                kind: FragmentKind::Expression,
                parsed: &parsed
            },
            ..splice
        }
        .boolean_literal()
        .is_err()
    );
    for source in [
        "7",
        "-7",
        "\"true\"",
        "'t'",
        "b\"true\"",
        "(true)",
        "{true}",
    ] {
        let parsed = checked(ParsedSyntax::own(
            source,
            checked(super::super::expression::parse(source))?,
        ))?;
        assert!(
            occurrence(source, &parsed).boolean_literal().is_err(),
            "{source}"
        );
    }
    Ok(())
}
