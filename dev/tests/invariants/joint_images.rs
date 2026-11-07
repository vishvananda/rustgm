//! The canonical macro image keeps source adjacency, and one re-lex rebuilds it.
use super::canonical;
use crate::{
    support::{
        crates::exported_macros::{Delimiter, ImageBuilder, Macro, Row, TokenClass},
        edition::Edition,
    },
    syntax::token_tree::Forest,
};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn parse(text: &str) -> Result<Forest> {
    Forest::parse(text).map_err(|error| format!("{}: {}", error.offset, error.message).into())
}

/// The provider's image, after the consumer's authentication: lexing the image
/// once must rebuild the same image and rows, which also pass the layout check.
fn authenticated(body: &str) -> Result<(String, Vec<Row>)> {
    let (image, rows) = canonical(body, &parse(body)?)?;
    let rebuilt = canonical(&image, &parse(&image)?)?;
    assert_eq!(rebuilt, (image.clone(), rows.clone()), "{body}");
    Macro::new(
        "m".to_owned(),
        Edition::Rust2021,
        false,
        1,
        image.clone(),
        rows.clone(),
    )?;
    Ok((image, rows))
}

#[test]
fn adjacent_pieces_stay_adjacent_and_separated_pieces_keep_one_space() -> Result {
    for (body, expected) in [
        ("{ () => { $crate::f() }; }", "{ () => { $crate::f() }; }"),
        // Metavariables, adjacent or separated, and repetitions.
        ("{($x:expr)=>{$x+1};}", "{($x:expr)=>{$x+1};}"),
        (
            "{ ( $ x : ident ) => { $ x } }",
            "{ ( $ x : ident ) => { $ x } }",
        ),
        ("{($($e:expr),*)=>{$($e)+*}}", "{($($e:expr),*)=>{$($e)+*}}"),
        // Comments and runs of whitespace become one space, also between
        // punctuation that would otherwise be joint.
        ("{a/* c */b\t\n// line\n  c/**/d}", "{a b c d}"),
        ("{+/**/= <\n\n< -/* /* nested */ */>}", "{+ = < < - >}"),
        // Delimiters adjacent to tokens, and empty groups with or without space.
        ("{f(a)[b]{c}( )[]{ }}", "{f(a)[b]{c}( )[]{ }}"),
        ("{/* x */a /* y */}", "{ a }"),
        // Lifetimes against characters, next to delimiters.
        (
            "{'a 'b' ('c) ('d') 'static&'e}",
            "{'a 'b' ('c) ('d') 'static&'e}",
        ),
        // Raw strings keep inner spaces, comment text and quotes.
        (
            r##"{r#"a  /* b */ "c"#  br"x"   cr#"y"#}"##,
            r##"{r#"a  /* b */ "c"# br"x" cr#"y"#}"##,
        ),
        (
            "{r#fn 'r#a 1.0f32 1. 0x1u8 b'a' c\"z\"}",
            "{r#fn 'r#a 1.0f32 1. 0x1u8 b'a' c\"z\"}",
        ),
    ] {
        assert_eq!(authenticated(body)?.0, expected, "{body}");
    }
    let (_, rows) = authenticated("{f(a)}")?;
    assert_eq!(
        rows,
        [
            Row::Group {
                delimiter: Delimiter::Brace,
                rows: 3
            },
            Row::Token {
                class: TokenClass::Identifier,
                length: 1
            },
            Row::Group {
                delimiter: Delimiter::Parenthesis,
                rows: 1
            },
            Row::Token {
                class: TokenClass::Identifier,
                length: 1
            },
        ]
    );
    let doc = canonical("{/// doc\na}", &parse("{/// doc\na}")?);
    assert_eq!(
        doc,
        Err("doc comments in exported macro bodies are outside the macro metadata profile")
    );
    Ok(())
}

#[test]
fn layout_admits_only_adjacent_or_single_space_pieces() -> Result {
    let two = vec![
        Row::Group {
            delimiter: Delimiter::Brace,
            rows: 2,
        },
        Row::Token {
            class: TokenClass::Identifier,
            length: 1,
        },
        Row::Token {
            class: TokenClass::Identifier,
            length: 1,
        },
    ];
    let empty = vec![
        Row::Group {
            delimiter: Delimiter::Brace,
            rows: 1,
        },
        Row::Group {
            delimiter: Delimiter::Parenthesis,
            rows: 0,
        },
    ];
    let layout = |image: &str, rows: &[Row]| {
        Macro::new(
            "m".to_owned(),
            Edition::Rust2021,
            false,
            1,
            image.to_owned(),
            rows.to_vec(),
        )
        .is_ok()
    };
    for image in ["{a b}", "{ a b }", "{ a b}", "{ab}"] {
        assert!(layout(image, &two), "{image}");
    }
    for image in ["{a  b}", " {a b}", "{a b} ", "{a b  }", "{a\tb}", "{a\nb}"] {
        assert!(!layout(image, &two), "{image}");
    }
    for image in ["{()}", "{ ( ) }", "{( )}"] {
        assert!(layout(image, &empty), "{image}");
    }
    for image in ["{(  )}", "{()  }", "{  ()}"] {
        assert!(!layout(image, &empty), "{image}");
    }
    // The layout check has no lexer: `{ab}` passes it, but its re-lex is one
    // identifier, so the consumer's rebuilt rows differ and it is rejected.
    assert_ne!(canonical("{ab}", &parse("{ab}")?)?.1, two);
    let mut builder = ImageBuilder::default();
    assert_eq!(
        builder.open(Delimiter::Brace, true),
        Err("exported macro image cannot begin with a separator")
    );
    Ok(())
}
