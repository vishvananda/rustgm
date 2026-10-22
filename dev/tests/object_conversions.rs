//! Object erasure proves complete implementation inputs before retaining a conversion.
use crate::qualify as rustc;
use crate::semantic;
use crate::verified_regions as regions;

const CASES: &[(&str, &str, bool)] = &[
    (
        "shared",
        "trait T{}impl T for u8{}fn f(x:&u8)->&dyn T{x}",
        true,
    ),
    (
        "mutable",
        "trait T{}impl T for u8{}fn f(x:&mut u8)->&mut dyn T{x}",
        true,
    ),
    (
        "share",
        "trait T{}impl T for u8{}fn f(x:&mut u8)->&dyn T{x}",
        true,
    ),
    (
        "strengthen",
        "trait T{}impl T for u8{}fn f(x:&u8)->&mut dyn T{x}",
        false,
    ),
    ("missing", "trait T{}fn f(x:&u8)->&dyn T{x}", false),
    (
        "wrong-impl",
        "trait T{}impl T for u16{}fn f(x:&u8)->&dyn T{x}",
        false,
    ),
    (
        "raw",
        "trait T{}impl T for u8{}fn f(x:*const u8)->*const dyn T{x}",
        true,
    ),
    (
        "reference-raw",
        "trait T{}impl T for u8{}fn f(x:&u8)->*const dyn T{x}",
        true,
    ),
    ("generic", "trait T{}fn f<X:T>(x:&X)->&dyn T{x}", true),
    (
        "unsized-generic",
        "trait T{}fn f<X:T+?Sized>(x:&X)->&dyn T{x}",
        false,
    ),
    (
        "unsized-slice",
        "trait T{}impl T for [u8]{}fn f(x:&[u8])->&dyn T{x}",
        false,
    ),
    (
        "trait-argument",
        "trait T<A>{}impl T<u8> for u16{}fn f(x:&u16)->&dyn T<u8>{x}",
        true,
    ),
    (
        "trait-argument-wrong",
        "trait T<A>{}impl T<u8> for u16{}fn f(x:&u16)->&dyn T<u32>{x}",
        false,
    ),
    (
        "associated",
        "trait T{type A;}impl T for u8{type A=u16;}fn f(x:&u8)->&dyn T<A=u16>{x}",
        true,
    ),
    (
        "associated-wrong",
        "trait T{type A;}impl T for u8{type A=u16;}fn f(x:&u8)->&dyn T<A=u32>{x}",
        false,
    ),
    (
        "associated-generic",
        "trait T{type A;}fn f<X:T<A=u16>>(x:&X)->&dyn T<A=u16>{x}",
        true,
    ),
    (
        "associated-generic-wrong",
        "trait T{type A;}fn f<X:T<A=u16>>(x:&X)->&dyn T<A=u32>{x}",
        false,
    ),
    (
        "inherited",
        "trait P{type A;}trait T:P<A=u16>{}impl P for u8{type A=u16;}impl T for u8{}fn f(x:&u8)->&dyn T{x}",
        true,
    ),
    (
        "inherited-conflict",
        "trait P{type A;}trait T:P<A=u16>{}impl P for u8{type A=u16;}impl T for u8{}fn f(x:&u8)->&dyn T<A=u32>{x}",
        false,
    ),
    (
        "quantified",
        "trait T<'a>{}impl<'a>T<'a> for u8{}fn f(x:&u8)->&dyn for<'a>T<'a>{x}",
        true,
    ),
    (
        "quantified-static-only",
        "trait T<'a>{}impl T<'static> for u8{}fn f(x:&u8)->&dyn for<'a>T<'a>{x}",
        false,
    ),
    (
        "quantified-associated",
        "trait T<'a>{type A;}impl<'a>T<'a> for u8{type A=&'a u8;}fn f(x:&u8)->&dyn for<'a>T<'a,A=&'a u8>{x}",
        true,
    ),
    (
        "quantified-associated-wrong",
        "trait T<'a>{type A;}impl<'a>T<'a> for u8{type A=&'static u8;}fn f(x:&u8)->&dyn for<'a>T<'a,A=&'a u8>{x}",
        false,
    ),
    (
        "struct-tail",
        "trait T{}impl T for u8{}struct S<X:?Sized>{prefix:u8,tail:X}fn f(x:&S<u8>)->&S<dyn T>{x}",
        true,
    ),
    (
        "borrowed-value",
        "trait T{}impl<'a>T for &'a u8{}fn f<'a,'b>(x:&'a &'b u8)->&'a dyn T{x}",
        true,
    ),
    (
        "no-extra-dereference",
        "trait T{}impl T for u8{}fn f<'a>(x:&'a &u8)->&'a dyn T{x}",
        false,
    ),
    (
        "literal",
        "trait T{}impl T for u8{}fn f(){let _: &dyn T=&7u8;}",
        true,
    ),
    (
        "branch",
        "trait T{}impl T for u8{}impl T for u16{}fn f<'a>(x:&'a u8,y:&'a u16,b:bool)->&'a dyn T{if b{x}else{y}}",
        true,
    ),
    (
        "generic-result",
        "trait T{type A;}impl T for u8{type A=u16;}fn f<X:T>(x:&X)->&dyn T<A=X::A>{x}",
        true,
    ),
    (
        "inherited-quantified",
        "trait P<'a>{type A;}trait T:for<'a>P<'a>{}impl<'a>P<'a> for u8{type A=u16;}impl T for u8{}fn f(x:&u8)->&dyn T<A=u16>{x}",
        true,
    ),
    (
        "inherited-quantified-wrong",
        "trait P<'a>{type A;}trait T:for<'a>P<'a>{}impl<'a>P<'a> for u8{type A=&'a u8;}impl T for u8{}fn f(x:&u8)->&dyn T<A=&'static u8>{x}",
        false,
    ),
    (
        "inherited-root-capture",
        "trait P<'a,'b>{type A;}trait T<'a>:for<'b>P<'a,'b>{}impl<'a,'b>P<'a,'b> for u8{type A=&'a u8;}impl<'a>T<'a> for u8{}fn f(x:&u8)->&dyn for<'a>T<'a,A=&'a u8>{x}",
        true,
    ),
    (
        "inherited-root-wrong",
        "trait P<'a,'b>{type A;}trait T<'a>:for<'b>P<'a,'b>{}impl<'a,'b>P<'a,'b> for u8{type A=&'b u8;}impl<'a>T<'a> for u8{}fn f(x:&u8)->&dyn for<'a>T<'a,A=&'a u8>{x}",
        false,
    ),
];

#[test]
fn implementation_and_equality_goals_match_rustc() -> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, source, accepted) in CASES {
        rustc::qualify(&format!("object-convert-{name}"), source, accepted)?;
        let result = regions::prove(source);
        if result.is_ok() != accepted {
            failures.push(format!("{name}: {:?}", result.err()));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
fn concrete_and_object_types_survive_body_construction() -> Result<(), Box<dyn std::error::Error>> {
    use rustgm::{
        body,
        types::{Kind, unsizing::Conversion},
    };
    let source = "trait T{}impl T for u8{}fn f(x:&u8)->&dyn T{x}";
    let (report, facts) = semantic::analyze(source)?;
    let program = body::build(&report, &facts)?;
    let selected = program
        .bodies
        .iter()
        .flat_map(|body| &body.values)
        .filter_map(|value| match value.kind {
            body::ir::Rvalue::Unsize { conversion, .. } => Some(conversion),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(selected.len(), 1);
    let Conversion::Object { concrete, target } = selected[0] else {
        return Err("object conversion was classified as slice metadata".into());
    };
    assert!(matches!(
        facts.types.kind(concrete),
        Some(Kind::Primitive(rustgm::types::Primitive::U8))
    ));
    assert!(matches!(
        facts.types.kind(target),
        Some(Kind::Dynamic { .. })
    ));
    let contracts = body::regions::collect(&facts)?;
    rustgm::borrow::regions::check_bodies(
        &facts.types,
        &facts.lifetimes.variances,
        &contracts,
        &program,
    )?;
    Ok(())
}

#[test]
fn erased_referent_must_outlive_the_object_bound() -> Result<(), Box<dyn std::error::Error>> {
    for (name, bound, accepted) in [("borrowed", "'a", true), ("escape", "'static", false)] {
        let source = format!(
            "trait T{{}}impl<'b>T for &'b u8{{}}fn f<'a,'b>(x:&'a &'b u8)->&'a (dyn T+{bound}){{x}}"
        );
        rustc::qualify(&format!("object-region-{name}"), &source, accepted)?;
        let (report, facts) = semantic::analyze(&source)?;
        let program = rustgm::body::build(&report, &facts)?;
        let contracts = rustgm::body::regions::collect(&facts)?;
        let result = rustgm::borrow::regions::check_bodies(
            &facts.types,
            &facts.lifetimes.variances,
            &contracts,
            &program,
        );
        assert_eq!(result.is_ok(), accepted, "{name}: {:?}", result.err());
    }
    Ok(())
}
