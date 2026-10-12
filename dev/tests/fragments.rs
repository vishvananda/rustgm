use rustgm::{
    lex::{
        Lexer,
        token::{Kind as TokenKind, Punctuation as P, Token},
    },
    syntax::{
        ast::Kind,
        cursor::{Cursor, TokenPosition},
        expression::fragment::{self, FragmentKind},
    },
};
fn tokens(source: &str) -> Result<Vec<Token>, String> {
    let mut lexer = Lexer::new(source).map_err(|e| format!("{e:?}"))?;
    let mut result = Vec::new();
    while let Some(token) = lexer.next_token().map_err(|e| format!("{e:?}"))? {
        if !matches!(
            token.kind,
            TokenKind::Whitespace | TokenKind::LineComment | TokenKind::BlockComment
        ) {
            result.push(token);
        }
    }
    Ok(result)
}
#[test]
fn typed_token_input_does_not_rescan_intervening_source() -> Result<(), String> {
    let source = "1 @ 2";
    let input = vec![
        Token {
            kind: TokenKind::Integer,
            start: 0,
            end: 1,
        },
        Token {
            kind: TokenKind::Punctuation(P::Plus),
            start: 2,
            end: 3,
        },
        Token {
            kind: TokenKind::Integer,
            start: 4,
            end: 5,
        },
    ];
    let parsed =
        fragment::parse(source, &input, FragmentKind::Expression).map_err(|e| format!("{e:?}"))?;
    assert!(matches!(
        parsed.ast.node(parsed.ast.root).kind,
        Kind::Binary { op: P::Plus, .. }
    ));
    assert_eq!(
        parsed.position,
        TokenPosition {
            index: 3,
            offset: 0
        }
    );
    Ok(())
}
#[test]
fn fragments_stop_at_grammar_boundaries_and_preserve_compound_remainders() -> Result<(), String> {
    for (source, kind, index, offset) in [
        ("a+b, rest", FragmentKind::Expression, 3, 0),
        ("Vec<u8>> tail", FragmentKind::Type, 3, 1),
        ("Vec<Vec<u8>>, tail", FragmentKind::Type, 6, 0),
        ("A|B=>value", FragmentKind::Pattern, 3, 0),
        ("A|B", FragmentKind::PatternParameter, 1, 0),
        ("let x=1; next", FragmentKind::Statement, 4, 0),
        ("struct S; next", FragmentKind::Item, 3, 0),
        ("-4suffix, next", FragmentKind::Literal, 2, 0),
    ] {
        let input = tokens(source)?;
        let parsed =
            fragment::parse(source, &input, kind).map_err(|e| format!("{source}: {e:?}"))?;
        assert_eq!(parsed.position, TokenPosition { index, offset }, "{source}");
        assert!(parsed.ast.lookahead_high_water <= 3);
        assert!(parsed.ast.literals.is_empty());
    }
    Ok(())
}
#[test]
fn edition_specific_expression_fragment_is_a_parser_constraint() -> Result<(), String> {
    for source in ["_", "const {1}"] {
        let input = tokens(source)?;
        assert!(fragment::parse(source, &input, FragmentKind::Expression).is_ok());
        assert!(fragment::parse(source, &input, FragmentKind::Expression2021).is_err());
    }
    let source = "(const {1})";
    let input = tokens(source)?;
    assert!(fragment::parse(source, &input, FragmentKind::Expression2021).is_ok());
    Ok(())
}
#[test]
fn invalid_external_token_ranges_are_diagnosed_without_indexing_panics() {
    for (start, end) in [(0, 8), (1, 2), (2, 1)] {
        let input = [Token {
            kind: TokenKind::Identifier,
            start,
            end,
        }];
        let mut cursor = Cursor::from_tokens("é", &input);
        assert!(cursor.peek(0).is_err());
    }
}

#[test]
fn a_fragment_can_resume_from_a_previous_compound_token_split() -> Result<(), String> {
    let source = "Vec<u8>>= tail";
    let input = tokens(source)?;
    let first =
        fragment::parse(source, &input, FragmentKind::Type).map_err(|e| format!("{e:?}"))?;
    assert_eq!(
        first.position,
        TokenPosition {
            index: 3,
            offset: 1
        }
    );
    let mut cursor =
        Cursor::from_position(source, &input, first.position).map_err(|e| format!("{e:?}"))?;
    assert!(cursor.at(P::GreaterEqual).map_err(|e| format!("{e:?}"))?);
    cursor
        .split(P::Greater, P::Equal)
        .map_err(|e| format!("{e:?}"))?;
    assert_eq!(
        cursor.token_position().map_err(|e| format!("{e:?}"))?,
        TokenPosition {
            index: 3,
            offset: 2
        }
    );
    let position = cursor.token_position().map_err(|e| format!("{e:?}"))?;
    let mut resumed =
        Cursor::from_position(source, &input, position).map_err(|e| format!("{e:?}"))?;
    assert!(resumed.at(P::Equal).map_err(|e| format!("{e:?}"))?);
    Ok(())
}

#[test]
fn metadata_and_visibility_keep_their_own_typed_grammar() -> Result<(), String> {
    let source = "foo = 1 + 2, rest";
    let input = tokens(source)?;
    let parsed = fragment::metadata(source, &input, TokenPosition::default())
        .map_err(|e| format!("{e:?}"))?;
    assert_eq!(parsed.position.index, 5);
    assert_eq!(parsed.attribute.name(&parsed.forest, source), Some("foo"));
    assert!(parsed.expression.is_some());
    for (source, count) in [
        ("pub(crate) struct S;", 4),
        ("pub fn f(){}", 1),
        ("struct S;", 0),
    ] {
        let input = tokens(source)?;
        let parsed = fragment::visibility(source, &input, TokenPosition::default())
            .map_err(|e| format!("{e:?}"))?;
        assert_eq!(parsed.position.index, count);
    }
    Ok(())
}
