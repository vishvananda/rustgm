use rustgm::{
    lex::token::{Kind as TokenKind, Punctuation as P, Token},
    literal::{Value, storage::Storage},
    syntax::{
        ast::Kind,
        cursor::TokenPosition,
        expression::{
            self,
            fragment::{self, FragmentKind as F},
        },
        interpolation::{Nonterminal, ParsedSyntax, Splice},
        token_tree::Span,
    },
};

fn owned(source: &str) -> Result<ParsedSyntax, String> {
    let ast = expression::parse(source).map_err(|e| format!("{e:?}"))?;
    ParsedSyntax::own(source, ast).map_err(|e| format!("{e:?}"))
}
#[test]
fn expansion_handles_fit_ordinary_token_storage_and_diagnose_overflow() -> Result<(), String> {
    use rustgm::lex::token::ExpansionId;
    assert!(size_of::<TokenKind>() <= 4);
    assert!(size_of::<Token>() <= 12);
    for index in [0, 1, 255, 256, 65535, 65536, (1 << 24) - 1] {
        let id = ExpansionId::new(index).map_err(|e| format!("{e:?}"))?;
        assert_eq!(id.get(), index);
    }
    assert!(ExpansionId::new(1 << 24).is_err());
    assert!(ExpansionId::new(u32::MAX).is_err());
    Ok(())
}
#[test]
fn opaque_expression_preserves_precedence_without_reading_its_spelling() -> Result<(), String> {
    let parsed = owned("1+2")?;
    let source = "??? * 3";
    let splices = [Splice {
        fragment: Nonterminal::Syntax {
            kind: F::Expression,
            parsed: &parsed,
        },
        original_source: "1+2",
        original: Span { start: 0, end: 3 },
        destination: Span { start: 0, end: 3 },
    }];
    let tokens = [
        Token {
            kind: TokenKind::interpolated(0).map_err(|e| format!("{e:?}"))?,
            start: 0,
            end: 3,
        },
        Token {
            kind: TokenKind::Punctuation(P::Star),
            start: 4,
            end: 5,
        },
        Token {
            kind: TokenKind::Integer,
            start: 6,
            end: 7,
        },
    ];
    let result = fragment::parse_with_splices(
        source,
        &tokens,
        F::Expression,
        TokenPosition::default(),
        &splices,
    )
    .map_err(|e| format!("{e:?}"))?;
    let Kind::Binary {
        op: P::Star, left, ..
    } = result.ast.node(result.ast.root).kind
    else {
        return Err("expected multiplication root".into());
    };
    assert!(matches!(
        result.ast.node(left).kind,
        Kind::Binary { op: P::Plus, .. }
    ));
    assert_eq!(result.ast.literals.len(), 2);
    assert_eq!(result.position.index, 3);
    assert!(
        fragment::parse_with_splices(source, &tokens, F::Type, TokenPosition::default(), &splices)
            .is_err()
    );
    Ok(())
}
#[test]
fn borrowed_and_decoded_literals_are_transferred_by_value() -> Result<(), String> {
    for original in [
        "\"plain\"",
        "\"escaped\\n\"",
        "0.25f32",
        "b\"\\xff\"",
        "c\"hello\"",
    ] {
        let parsed = owned(original)?;
        let source = format!("prefix {original}");
        let size = u32::try_from(original.len()).map_err(|e| e.to_string())?;
        let splices = [Splice {
            fragment: Nonterminal::Syntax {
                kind: F::Expression,
                parsed: &parsed,
            },
            original_source: original,
            original: Span {
                start: 0,
                end: size,
            },
            destination: Span {
                start: 7,
                end: 7 + size,
            },
        }];
        let tokens = [Token {
            kind: TokenKind::interpolated(0).map_err(|e| format!("{e:?}"))?,
            start: 7,
            end: 7 + size,
        }];
        let result = fragment::parse_with_splices(
            &source,
            &tokens,
            F::Expression,
            TokenPosition::default(),
            &splices,
        )
        .map_err(|e| format!("{original}: {e:?}"))?;
        let expected = parsed
            .literals
            .view(original, parsed.ast.literals[0])
            .map_err(str::to_owned)?;
        assert_eq!(result.ast.literals[0], expected);
        let mut storage = Storage::default();
        let value = storage
            .store(&source, result.ast.literals[0].clone())
            .map_err(str::to_owned)?;
        assert_eq!(
            storage.view(&source, value).map_err(str::to_owned)?,
            expected
        );
        if original == "\"plain\"" {
            assert!(matches!(
                result.ast.literals[0],
                Value::String(std::borrow::Cow::Borrowed(_))
            ));
        }
    }
    Ok(())
}
#[test]
fn malformed_opaque_handles_and_provenance_are_diagnosed() -> Result<(), String> {
    let parsed = owned("1+2")?;
    let splices = [Splice {
        fragment: Nonterminal::Syntax {
            kind: F::Expression,
            parsed: &parsed,
        },
        original_source: "1+2",
        original: Span { start: 0, end: 3 },
        destination: Span { start: 0, end: 2 },
    }];
    for (index, end) in [(1, 2), (0, 3), (0, 2)] {
        let tokens = [Token {
            kind: TokenKind::interpolated(index).map_err(|e| format!("{e:?}"))?,
            start: 0,
            end,
        }];
        assert!(
            fragment::parse_with_splices(
                "1+2",
                &tokens,
                F::Expression,
                TokenPosition::default(),
                &splices
            )
            .is_err()
        );
    }
    Ok(())
}
