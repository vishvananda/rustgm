#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;
use rustgm::types::{Kind, Primitive, TypeId, arguments::Argument};
use std::fmt::Write as _;

const ACCEPTED: &[(&str, &str)] = &[
    ("self-explicit", "type A<T=A<u8>>=T;fn f(x:A)->u8{x}"),
    (
        "self-dependent-later",
        "type A<T=A<u8>,U=T>=(T,U);fn f(x:A)->((u8,u8),(u8,u8)){x}",
    ),
    (
        "self-fixed-later",
        "type A<T=A<u8>,const N:usize=3>=(T,[u8;N]);fn f(x:A)->((u8,[u8;3]),[u8;3]){x}",
    ),
    (
        "self-explicit-later",
        "type A<T=A<u8,4>,const N:usize=3>=(T,[u8;N]);fn f(x:A)->((u8,[u8;4]),[u8;3]){x}",
    ),
    (
        "mutual-explicit",
        "type A<T=B<u8>>=T;type B<U=A<u8>>=U;fn f(x:A)->B{x}",
    ),
    (
        "body-before-default",
        "type A<T=u8>=B<T>;type B<U=A>=U;fn f(x:A)->B{x}",
    ),
    (
        "default-through-body",
        "type A<T=B>=T;type B=A<u16>;fn f(x:A)->u16{x}",
    ),
    (
        "self-bound",
        "type A<T> where A<T>:Copy = T;fn f(x:A<u8>)->u8{x}",
    ),
    (
        "self-bound-default",
        "type A<T=A<u8>> where A<T>:Copy = T;fn f(x:A)->u8{x}",
    ),
    (
        "mutual-bounds",
        "type A<T> where B<T>:Copy =T;type B<U> where A<U>:Clone =U;fn f(x:A<u8>)->B<u8>{x}",
    ),
    (
        "type-and-const-alias",
        "type Size=usize;type A<T=A<u8>,const N:Size=3>=(T,[u8;N]);fn f(x:A)->((u8,[u8;3]),[u8;3]){x}",
    ),
    (
        "lifetime-self",
        "type A<'a,T=A<'a,&'a u8>>=T;fn f<'a>(x:A<'a>)->&'a u8{x}",
    ),
    (
        "late-const-name",
        "type A<T=A<u8>,const N:usize=C>=(T,[u8;N]);const C:usize=3;fn f(x:A)->((u8,[u8;3]),[u8;3]){x}",
    ),
    (
        "const-expression-default",
        "type A<T=A<u8>,const N:usize={1+2}>=(T,[u8;N]);fn f(x:A)->((u8,[u8;3]),[u8;3]){x}",
    ),
    (
        "independent-owner-default",
        "type A<T=A<u8>>=T;type B<T=A<u16>>=T;fn f(x:A,y:B)->(u8,u16){(x,y)}",
    ),
    (
        "higher-ranked-self",
        "type A<T=A<for<'a>fn(&'a u8)->&'a u8>>=T;fn f(x:A)->for<'a>fn(&'a u8)->&'a u8{x}",
    ),
    (
        "configured-default",
        "type A<#[cfg(any())] Gone=u8,T=A<u16>>=T;fn f(x:A)->u16{x}",
    ),
    (
        "macro-default",
        "macro_rules! m{($name:ident)=>{type $name<T=$name<u8>>=T;}}m!(A);fn f(x:A)->u8{x}",
    ),
];
const REJECTED: &[(&str, &str)] = &[
    ("default-cycle", "type A<T=A>=T;"),
    ("mutual-default-cycle", "type A<T=B>=T;type B<U=A>=U;"),
    ("default-body-cycle", "type A<T=B>=T;type B=A;"),
    ("body-cycle", "type A<T=u8>=*const A<T>;"),
    ("mutual-body-cycle", "type A<T=u8>=B<T>;type B<U=u8>=A<U>;"),
    ("self-parameter", "type A<T=T>=T;"),
    ("forward-parameter", "type A<T=U,U=u8>=(T,U);"),
    ("self-const", "type A<const N:usize=N>=[u8;N];"),
    (
        "forward-const",
        "type A<const M:usize=N,const N:usize=3>=([u8;M],[u8;N]);",
    ),
    ("forward-through-self", "type A<T=A<U>,U=u8>=(T,U);"),
    (
        "unused-invalid-const",
        "type A<const N:u8=256>=u8;fn f(_:A<3>){}",
    ),
    (
        "unused-missing-default-argument",
        "type A<T=B>=T;type B<U>=U;",
    ),
    (
        "unused-kind-mismatch",
        "type A<T=B<true>>=T;type B<const N:usize>=[u8;N];",
    ),
    ("default-bound-unknown", "type A<T> where A<T>:Missing=T;"),
];

#[test]
fn finite_defaults_and_bounds_can_demand_completed_alias_bodies() -> Result<(), String> {
    for &(name, source) in ACCEPTED {
        semantic::analyze(source).map_err(|e| format!("{name}: {e}"))?;
    }
    Ok(())
}
#[test]
fn unused_defaults_and_actual_expansion_cycles_remain_errors() {
    for &(name, source) in REJECTED {
        assert!(semantic::analyze(source).is_err(), "{name}: {source}");
    }
}
#[test]
fn default_readiness_outcomes_match_independent_rustc() -> Result<(), Box<dyn std::error::Error>> {
    for (cases, accepted) in [(ACCEPTED, true), (REJECTED, false)] {
        for &(name, source) in cases {
            rustc::qualify(&format!("default-readiness-{name}"), source, accepted)?;
        }
    }
    Ok(())
}
#[test]
fn default_templates_are_substituted_with_the_supplied_prefix() -> Result<(), String> {
    let (_, facts) = semantic::analyze(
        "type A<T=A<u8>,U=T>=(T,U);fn f(_:A){}fn g(_:((u8,u8),(u8,u8))){}fn h(_:A<u16>){}fn i(_:(u16,u16)){}",
    )?;
    assert_eq!(facts.functions[0].signature, facts.functions[1].signature);
    assert_eq!(facts.functions[2].signature, facts.functions[3].signature);
    assert_ne!(facts.functions[0].signature, facts.functions[2].signature);
    let parameters = facts
        .generics
        .parameters
        .values()
        .next()
        .ok_or("missing alias parameters")?;
    let Some(Argument::Type(second)) = parameters[1].default else {
        return Err("missing dependent default template".into());
    };
    assert_eq!(parameters[0].argument, Argument::Type(second));
    let Some(Argument::Type(first)) = parameters[0].default else {
        return Err("missing first default".into());
    };
    let Some(Kind::Tuple(list)) = facts.types.kind(first) else {
        return Err("first default is not a closed pair".into());
    };
    assert_eq!(
        facts.types.list(list),
        Some([TypeId::primitive(Primitive::U8); 2].as_slice())
    );
    Ok(())
}
#[test]
fn forward_default_chains_are_iterative_and_shared() -> Result<(), String> {
    let mut source = String::new();
    for index in 0..2048 {
        write!(source, "type A{index}<T=A{}>=T;", index + 1).map_err(|e| e.to_string())?;
    }
    source.push_str("type A2048<T=u8>=T;fn f(_:A0){}fn g(_:u8){}");
    let (_, facts) = semantic::analyze(&source)?;
    assert_eq!(facts.functions[0].signature, facts.functions[1].signature);
    for parameters in facts.generics.parameters.values() {
        assert_eq!(
            parameters[0].default,
            Some(Argument::Type(TypeId::primitive(Primitive::U8)))
        );
    }
    Ok(())
}

#[test]
fn default_cycles_have_their_own_query_diagnostic() {
    for (source, diagnostic) in [
        ("type A<T=A>=T;", "recursive generic default"),
        ("type A<T=B>=T;type B<U=A>=U;", "recursive generic default"),
        ("type A<T=u8>=*const A<T>;", "recursive type alias"),
    ] {
        let error = semantic::analyze(source).err().unwrap_or_default();
        assert!(error.contains(diagnostic), "{source}: {error}");
    }
}
#[test]
fn wide_dependent_defaults_reuse_one_argument_environment() -> Result<(), String> {
    let mut source = String::from("type A<T0=u8");
    for index in 1..2048 {
        write!(source, ",T{index}=T{}", index - 1).map_err(|e| e.to_string())?;
    }
    source.push_str(">=(");
    for index in 0..2048 {
        write!(source, "T{index},").map_err(|e| e.to_string())?;
    }
    source.push_str(");fn f(_:A){}fn g(_:(");
    for _ in 0..2048 {
        source.push_str("u8,");
    }
    source.push_str(")){}");
    let (_, facts) = semantic::analyze(&source)?;
    assert_eq!(facts.functions[0].signature, facts.functions[1].signature);
    Ok(())
}
