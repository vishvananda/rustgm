use rustgm::{
    lex::{
        Lexer, render,
        token::{Kind, Token},
    },
    support::source::SourceFile,
};

fn scan(text: &str) -> Result<Vec<Token>, String> {
    let mut lexer = Lexer::new(text).map_err(|error| format!("{error:?}"))?;
    let mut tokens = Vec::new();
    while let Some(token) = lexer.next_token().map_err(|error| format!("{error:?}"))? {
        assert!(token.end > token.start);
        assert!(text.is_char_boundary(token.start as usize));
        assert!(text.is_char_boundary(token.end as usize));
        tokens.push(token);
        assert!(tokens.len() <= text.len());
    }
    assert_eq!(lexer.next_token(), Ok(None));
    Ok(tokens)
}

#[test]
fn hand_checked_text_contract() -> Result<(), String> {
    let mut output = Vec::new();
    render::tokens("fn x()", &mut output).map_err(|error| format!("{error:?}"))?;
    assert_eq!(output, b"identifier 0 2 \"fn\"\nwhitespace 2 3 \" \"\nidentifier 3 4 \"x\"\npunctuation 4 5 \"(\"\npunctuation 5 6 \")\"\neof 6\n");
    Ok(())
}

#[test]
fn source_normalization_is_once_and_does_not_append_newline() -> Result<(), String> {
    let source = SourceFile::new(b"a\r\r\n\nb".to_vec())?;
    assert_eq!(source.original(), "a\r\r\n\nb");
    assert_eq!(source.text(), "a\r\n\nb");
    let plain = SourceFile::new(b"x".to_vec())?;
    assert_eq!(plain.text().as_ptr(), plain.original().as_ptr());
    assert!(SourceFile::new(vec![0xff]).is_err());
    Ok(())
}

#[test]
fn macro_literals_keep_arbitrary_suffixes() -> Result<(), String> {
    let tokens = scan("\"x\"suffix 12suffix r#\"x\"#suffix")?;
    assert_eq!(
        tokens.iter().map(|token| token.kind).collect::<Vec<_>>(),
        [
            Kind::String,
            Kind::Whitespace,
            Kind::Integer,
            Kind::Whitespace,
            Kind::RawString
        ]
    );
    Ok(())
}

#[test]
fn lifetime_and_character_boundaries() -> Result<(), String> {
    let tokens = scan("'a 'a' 'r#type b'x'")?;
    assert_eq!(
        tokens.iter().map(|token| token.kind).collect::<Vec<_>>(),
        [
            Kind::Lifetime,
            Kind::Whitespace,
            Kind::Character,
            Kind::Whitespace,
            Kind::RawLifetime,
            Kind::Whitespace,
            Kind::Byte
        ]
    );
    Ok(())
}

#[test]
fn nested_comments_are_iterative_and_compact() -> Result<(), String> {
    let source = format!("{}{}", "/*".repeat(20000), "*/".repeat(20000));
    assert_eq!(scan(&source)?.len(), 1);
    assert!(size_of::<Token>() <= 16);
    Ok(())
}

#[test]
fn shebang_probe_retains_inner_attributes() -> Result<(), String> {
    let tokens = scan("#! /* x */ [allow(x)]")?;
    assert_eq!(tokens[0].start, 0);
    let tokens = scan("\u{feff}#!/bin/rust\nx")?;
    assert_eq!(
        tokens.last().map(|token| token.kind),
        Some(Kind::Identifier)
    );
    assert!(tokens[0].start >= 3);
    Ok(())
}

#[test]
fn invalid_token_keeps_complete_output_prefix() {
    let mut output = Vec::new();
    let result = render::tokens("x /*", &mut output);
    assert!(result.is_err());
    assert_eq!(output, b"identifier 0 1 \"x\"\nwhitespace 1 2 \" \"\n");
}

#[test]
fn scalar_corpus_always_advances_or_diagnoses() -> Result<(), String> {
    let alphabet = [
        'a', '\u{301}', '\u{200e}', '🦀', '9', '\'', '"', '#', '/', '*', '\\', '\n', '_',
    ];
    let mut seed = 42_u64;
    for _ in 0..1000 {
        let mut text = String::new();
        for _ in 0..64 {
            seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            let index = usize::try_from(seed % 13).map_err(|error| error.to_string())?;
            text.push(alphabet[index]);
        }
        // Both outcomes are legitimate for this deliberately malformed corpus.
        // scan checks progress, span validity and bounded token count on success.
        let _ = scan(&text);
    }
    Ok(())
}
