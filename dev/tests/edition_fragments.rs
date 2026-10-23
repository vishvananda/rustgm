//! PA34 defining-edition fragment selection. Independent cross-crate rustc
//! consumers qualify the rule outcomes; driver/provenance integration is pending.
use rustgm::{
    expand::macros::{
        compile,
        input::Input,
        matcher::{Compiled, Matched},
    },
    support::{edition::Edition, names::Names},
    syntax::token_tree::Forest,
};

fn invoke(edition: Edition, rules: &str, source: &str) -> Result<Matched, String> {
    let mut names = Names::default();
    let forest = Forest::parse(rules).map_err(|error| format!("{error:?}"))?;
    let definition = compile::in_edition(rules, &forest, &mut names, edition)
        .map_err(|error| format!("{error:?}"))?;
    let forest = Forest::parse(source).map_err(|error| format!("{error:?}"))?;
    let input =
        Input::from_forest(&forest, forest.roots()).map_err(|error| format!("{error:?}"))?;
    Compiled::new(definition)
        .invoke(rules, source, &input, &mut names)
        .map_err(|error| format!("{error:?}"))
}

#[test]
fn defining_edition_selects_top_level_expression_grammar() -> Result<(), String> {
    let rules = "{($e:expr)=>{}; (const $e:expr)=>{}; (_)=>{};}";
    for edition in [Edition::Rust2021, Edition::Rust2024] {
        for source in ["7", "{ const { 7 } }", "(const { 7 })"] {
            assert_eq!(invoke(edition, rules, source)?.rule, 0);
        }
        let (constant, underscore) = match edition {
            Edition::Rust2021 => (1, 2),
            Edition::Rust2024 => (0, 0),
        };
        assert_eq!(invoke(edition, rules, "const { 7 }")?.rule, constant);
        assert_eq!(invoke(edition, rules, "_")?.rule, underscore);
    }
    Ok(())
}

#[test]
fn explicit_old_expression_grammar_is_edition_independent() -> Result<(), String> {
    let rules = "{($e:expr_2021)=>{}; (const $e:expr_2021)=>{}; (_)=>{};}";
    for edition in [Edition::Rust2021, Edition::Rust2024] {
        assert_eq!(invoke(edition, rules, "7")?.rule, 0);
        assert_eq!(invoke(edition, rules, "const { 7 }")?.rule, 1);
        assert_eq!(invoke(edition, rules, "_")?.rule, 2);
    }
    Ok(())
}

#[test]
fn unmatched_old_expression_does_not_accept_new_top_level_forms() {
    for source in ["const { 7 }", "_"] {
        let rules = "{($e:expr)=>{}}";
        assert!(invoke(Edition::Rust2021, rules, source).is_err());
        assert!(invoke(Edition::Rust2024, rules, source).is_ok());
        for edition in [Edition::Rust2021, Edition::Rust2024] {
            assert!(invoke(edition, "{($e:expr_2021)=>{}}", source).is_err());
        }
    }
    for edition in [Edition::Rust2021, Edition::Rust2024] {
        // A committed expression parse error must not fall back to a tt rule.
        assert!(invoke(edition, "{($e:expr)=>{}; ($($t:tt)*)=>{};}", "1 +").is_err());
    }
}

#[test]
fn normalized_fragment_kind_preserves_capture_memoization() -> Result<(), String> {
    let rules = "{($e:expr, no)=>{}; ($e:expr_2021, yes)=>{};}";
    let matched = invoke(Edition::Rust2021, rules, "7 + 1, yes")?;
    assert_eq!(matched.rule, 1);
    assert_eq!(matched.captures.parser_calls, 1);
    assert_eq!(matched.captures.values.len(), 1);
    let matched = invoke(Edition::Rust2024, rules, "7 + 1, yes")?;
    assert_eq!(matched.rule, 1);
    assert_eq!(matched.captures.parser_calls, 2);
    assert_eq!(matched.captures.values.len(), 2);
    Ok(())
}
