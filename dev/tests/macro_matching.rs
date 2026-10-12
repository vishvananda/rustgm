use rustgm::{
    expand::macros::{
        compile,
        input::Input,
        matcher::{Compiled, Matched},
    },
    support::names::Names,
    syntax::token_tree::Forest,
};

fn invoke(rules: &str, source: &str) -> Result<Matched, String> {
    let mut names = Names::default();
    let forest = Forest::parse(rules).map_err(|e| format!("{e:?}"))?;
    let definition = compile::compile(rules, &forest, &mut names).map_err(|e| format!("{e:?}"))?;
    let forest = Forest::parse(source).map_err(|e| format!("{e:?}"))?;
    let input = Input::from_forest(&forest, forest.roots()).map_err(|e| format!("{e:?}"))?;
    Compiled::new(definition)
        .invoke(rules, source, &input, &mut names)
        .map_err(|e| format!("{e:?}"))
}

#[test]
fn ordered_rules_reuse_fragments_from_failed_alternatives() -> Result<(), String> {
    let matched = invoke(
        "{($e:expr, nope)=>{}; ($e:expr, yes)=>{}; ($($t:tt)*)=>{};}",
        "a + 1, yes",
    )?;
    assert_eq!(matched.rule, 1);
    assert_eq!(matched.captures.parser_calls, 1);
    assert_eq!(matched.captures.values.len(), 1);
    assert_eq!(invoke("{(one)=>{};($i:ident)=>{}}", "one")?.rule, 0);
    Ok(())
}

#[test]
fn nested_and_optional_repetitions_match_without_recursive_backtracking() -> Result<(), String> {
    for (rules, source) in [
        ("{($($i:ident),*)=>{}}", ""),
        ("{($($i:ident),*)=>{}}", "a,b,c"),
        ("{($($i:ident),+ $(,)?)=>{}}", "a,b,c,"),
        ("{($([$( $i:ident ),*]);*)=>{}}", "[a,b];[];[c]"),
        ("{($($i:ident)* ; $j:ident)=>{}}", "a b; c"),
        ("{($($t:tt)*)=>{}}", "[a(b)] {c} !"),
        ("{($v:vis struct $i:ident)=>{}}", "struct S"),
        ("{($v:vis struct $i:ident)=>{}}", "pub(crate) struct S"),
        ("{(Vec<$t:ty>)=>{}}", "Vec<Vec<u8>>"),
    ] {
        invoke(rules, source)?;
    }
    let source = "x,".repeat(10_000);
    let result = invoke("{($($i:ident,)*)=>{}}", &source)?;
    assert!(result.work < source.len() * 20);
    assert_eq!(result.captures.values.len(), 10_000);
    Ok(())
}

#[test]
fn local_ambiguity_is_reported_before_speculative_fragment_parsing() {
    for (rules, source) in [
        ("{($($i:ident)* $j:ident)=>{}}", "x"),
        ("{($($i:ident)* literal)=>{}}", "literal"),
        ("{($($i:ident)? $j:ident)=>{}}", "x"),
    ] {
        let result = invoke(rules, source);
        assert!(
            result.is_err_and(|e| e.contains("local ambiguity")),
            "{rules}"
        );
    }
}

#[test]
fn literal_spelling_and_raw_identifier_flags_are_part_of_matching() -> Result<(), String> {
    for (rules, source) in [
        ("{(foo)=>{}}", "r#foo"),
        ("{(r#foo)=>{}}", "foo"),
        ("{(16)=>{}}", "0x10"),
        ("{($i:ident)=>{}}", "_"),
        ("{($v:vis)=>{}}", ""),
        ("{(($v:vis))=>{}}", "()"),
        ("{($($i:ident),+)=>{}}", ""),
        ("{($($i:ident),*)=>{}}", "a,"),
        ("{(())=>{}}", "{}"),
    ] {
        assert!(invoke(rules, source).is_err(), "{rules}: {source}");
    }
    assert_eq!(invoke("{(é)=>{};($i:ident)=>{}}", "e\u{301}")?.rule, 0);
    assert_eq!(invoke("{($v:vis)=>{};()=>{}}", "")?.rule, 1);
    assert_eq!(invoke("{($v:vis ,)=>{}; (1)=>{}}", "1")?.rule, 1);
    Ok(())
}

#[test]
fn a_selected_fragment_parser_failure_does_not_try_later_rules() {
    for (rules, source) in [
        ("{($e:expr)=>{};($($t:tt)*)=>{}}", "1 +"),
        ("{($i:item)=>{};($($t:tt)*)=>{}}", "1"),
        ("{($p:path)=>{};($($t:tt)*)=>{}}", "if"),
    ] {
        assert!(invoke(rules, source).is_err(), "{source}");
    }
}
