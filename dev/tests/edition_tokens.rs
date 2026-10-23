//! Token edition is independent of the macro definition's fragment grammar.
use rustgm::{
    expand::macros::{
        compile,
        input::Input,
        matcher::{Compiled, Workspace},
    },
    lex::token::LexError,
    support::{edition::Edition, names::Names},
    syntax::{
        cursor::TokenPosition,
        edition_context::WithEditions,
        expression::{fragment, parse_crate, parse_crate_in_edition},
        interpolation::EMPTY_INPUTS,
        token_tree::{Forest, error},
    },
};
use std::cell::Cell;

fn input(source: &str) -> Result<Input, LexError> {
    let forest = Forest::parse(source)?;
    Input::from_forest(&forest, forest.roots())
}

#[test]
fn source_identifier_positions_observe_their_explicit_edition() {
    let sources = [
        "fn gen() {} fn main() { gen(); }",
        "struct S { gen: u8 } fn f(s: S) { let S { gen } = s; let _ = gen; }",
        "mod gen { pub fn f() {} } use gen::f;",
        "fn f<gen>(x: gen) -> gen { x }",
        "fn f<r#gen>() -> impl Sized + use<gen> { 0 }",
        "#[cfg(any())] #[gen] fn f() {}",
    ];
    for source in sources {
        assert!(
            parse_crate_in_edition(source, None, Edition::Rust2021).is_ok(),
            "{source}"
        );
        assert!(
            parse_crate_in_edition(source, None, Edition::Rust2024).is_err(),
            "{source}"
        );
        assert!(
            parse_crate(source).is_err(),
            "default must stay Rust 2024: {source}"
        );
    }
    for edition in [Edition::Rust2021, Edition::Rust2024] {
        assert!(
            parse_crate_in_edition("fn r#gen() {} fn main() { r#gen(); }", None, edition).is_ok()
        );
        assert!(parse_crate_in_edition("fn yield() {}", None, edition).is_err());
    }
}

#[test]
fn reused_matcher_uses_invocation_token_edition_without_reusing_capture_permission()
-> Result<(), LexError> {
    let rules = "{($e:expr)=>{$e}}";
    for defining in [Edition::Rust2021, Edition::Rust2024] {
        let mut names = Names::default();
        let definition = compile::in_edition(rules, &Forest::parse(rules)?, &mut names, defining)?;
        let mut compiled = Compiled::new(definition);
        let mut workspace = Workspace::default();
        let input = input("gen()")?;
        for caller in [Edition::Rust2021, Edition::Rust2024, Edition::Rust2021] {
            let result = compiled.invoke_with_inputs(
                rules,
                "gen()",
                &input,
                &mut names,
                &mut workspace,
                &caller,
            );
            assert_eq!(result.is_ok(), caller == Edition::Rust2021);
            if let Ok(matched) = result {
                assert_eq!(matched.captures.parser_calls, 1);
            }
        }
    }
    Ok(())
}

#[test]
fn metadata_and_restricted_visibility_demand_token_edition() -> Result<(), LexError> {
    let metadata = input("gen")?;
    let visibility = input("pub(in gen)")?;
    for edition in [Edition::Rust2021, Edition::Rust2024] {
        assert_eq!(
            fragment::metadata_with_inputs(
                "gen",
                &metadata.tokens,
                TokenPosition::default(),
                &edition,
            )
            .is_ok(),
            edition == Edition::Rust2021
        );
        assert_eq!(
            fragment::visibility_with_inputs(
                "pub(in gen)",
                &visibility.tokens,
                TokenPosition::default(),
                &edition,
            )
            .is_ok(),
            edition == Edition::Rust2021
        );
    }
    Ok(())
}

#[test]
fn ordinary_and_raw_tokens_do_not_query_edition_provenance() -> Result<(), LexError> {
    let queries = Cell::new(0);
    let context = WithEditions::new(&EMPTY_INPUTS, |token| {
        queries.set(queries.get() + 1);
        Err(error(token.start, "missing test edition provenance"))
    });
    for source in ["ordinary + other", "r#gen()"] {
        let input = input(source)?;
        fragment::parse_with_inputs(
            source,
            &input.tokens,
            fragment::FragmentKind::Expression,
            TokenPosition::default(),
            &context,
        )?;
    }
    assert_eq!(queries.get(), 0);
    let input = input("gen()")?;
    let result = fragment::parse_with_inputs(
        "gen()",
        &input.tokens,
        fragment::FragmentKind::Expression,
        TokenPosition::default(),
        &context,
    );
    assert!(result.is_err());
    assert_eq!(queries.get(), 1);
    Ok(())
}

#[test]
fn transparent_capture_does_not_interpret_reserved_identifiers() -> Result<(), LexError> {
    let context = WithEditions::new(&EMPTY_INPUTS, |token| {
        Err(error(
            token.start,
            "transparent capture must not demand an edition",
        ))
    });
    for rules in ["{($i:ident)=>{}}", "{($t:tt)=>{}}"] {
        let mut names = Names::default();
        let definition = compile::compile(rules, &Forest::parse(rules)?, &mut names)?;
        Compiled::new(definition).invoke_with_inputs(
            rules,
            "gen",
            &input("gen")?,
            &mut names,
            &mut Workspace::default(),
            &context,
        )?;
    }
    Ok(())
}
