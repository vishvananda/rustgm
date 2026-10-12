use rustgm::{
    expand::macros::{
        compile,
        model::{Fragment, Kind, Repetition},
    },
    support::names::Names,
    syntax::token_tree::Forest,
};
fn definition(source: &str) -> Result<rustgm::expand::macros::model::Definition, String> {
    let forest = Forest::parse(source).map_err(|e| format!("{e:?}"))?;
    compile::compile(source, &forest, &mut Names::default()).map_err(|e| format!("{e:?}"))
}
#[test]
fn definitions_retain_rule_order_groups_and_repetition_owners() -> Result<(), String> {
    let d = definition("{(one)=>{1};($($name:ident),+)=>{[$($name),*]};}")?;
    assert_eq!(d.rules.len(), 2);
    assert_eq!(d.rules[1].variables.len(), 1);
    let v = &d.rules[1].variables[0];
    assert_eq!(v.fragment, Fragment::Identifier);
    assert_eq!(d.list(v.repetitions).len(), 1);
    assert!(matches!(
        d.node(d.list(v.repetitions)[0]).kind,
        Kind::Repeat {
            operator: Repetition::OneOrMore,
            ..
        }
    ));
    Ok(())
}
#[test]
fn malformed_rules_and_nullable_repetitions_fail_before_invocation() {
    for source in [
        "{}",
        "{($x)=>{}}",
        "{($x:no_such_fragment)=>{}}",
        "{($x:ident,$x:ident)=>{}}",
        "{($(a),?)=>{}}",
        "{($()*)=>{}}",
        "{($($v:vis)*)=>{}}",
        "{($($v:vis)?)=>{}}",
        "{($x:)=>{}}",
        "{($($x:) ident)=>{}}",
        "{(a){}}",
        "{(a)=>{}(b)=>{}}",
    ] {
        assert!(definition(source).is_err(), "{source}");
    }
}
#[test]
fn follow_sets_include_nullable_repetitions_and_nested_group_boundaries() {
    for source in [
        "{($e:expr,)=>{}}",
        "{($)=>{}}",
        "{($($e:expr)*)=>{}}",
        "{($($e:expr, $f:expr)*)=>{}}",
        "{($e:expr $(,$i:ident)*)=>{}}",
        "{(($e:expr) x)=>{}}",
        "{($p:pat if $e:expr)=>{}}",
        "{($p:pat_param | $q:pat_param)=>{}}",
        "{($t:ty $b:block)=>{}}",
        "{($v:vis struct $i:ident)=>{}}",
        "{($v:vis 'a)=>{}}",
        "{($($e:expr),*)=>{}}",
        "{($($i:ident)* ; $e:expr)=>{}}",
    ] {
        assert!(definition(source).is_ok(), "{source}");
    }
    for source in [
        "{($e:expr x)=>{}}",
        "{($e:expr [])=>{}}",
        "{($p:pat |)=>{}}",
        "{($e:expr $(,)? x)=>{}}",
        "{($v:vis priv)=>{}}",
        "{($v:vis $e:expr)=>{}}",
        "{($t:ty +)=>{}}",
    ] {
        assert!(definition(source).is_err(), "{source}");
    }
}
#[test]
fn many_ordered_rules_use_a_shared_fact_arena() -> Result<(), String> {
    let source = format!("{{{}}}", "($e:expr)=>{$e};".repeat(10_000));
    let d = definition(&source)?;
    assert_eq!(d.rules.len(), 10_000);
    assert_eq!(d.nodes.len(), 20_000);
    Ok(())
}
