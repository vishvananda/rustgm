//! Object membership supplies typed trait evidence without a concrete implementation.
use crate::qualify as rustc;
use crate::semantic;

const CASES: &[(&str, &str, bool)] = &[
    (
        "quantified-membership",
        "trait T<'a>{}fn need<X:for<'a>T<'a>+?Sized>(_:&X){}fn f(x:&dyn for<'a>T<'a>){need(x)}",
        true,
    ),
    (
        "static-membership",
        "trait T<'a>{}fn need<X:for<'a>T<'a>+?Sized>(_:&X){}fn f(x:&dyn T<'static>){need(x)}",
        false,
    ),
    (
        "inherited-quantified-equality",
        "trait P<'a>{type A;fn get(&self,x:&'a u8)->Self::A;}trait T:for<'a>P<'a,A=&'a u8>{}fn f<'a>(x:&dyn T,y:&'a u8)->&'a u8{x.get(y)}",
        true,
    ),
    (
        "quantified-result-escape",
        "trait T<'a>{type A;fn get(&self,x:&'a u8)->Self::A;}fn f(x:&dyn for<'b>T<'b,A=&'b u8>,y:&u8)->&'static u8{x.get(y)}",
        false,
    ),
    (
        "borrowed-result",
        "trait T{fn get(&self)->&u8;}fn f(x:&dyn T)->&u8{x.get()}",
        true,
    ),
    (
        "borrowed-result-escape",
        "trait T{fn get(&self)->&u8;}fn f(x:&dyn T)->&'static u8{x.get()}",
        false,
    ),
    (
        "diamond-result",
        "trait P{type A;fn get(&self)->Self::A;}trait L:P<A=u8>{}trait R:P<A=u8>{}trait T:L+R{}fn f(x:&dyn T)->u8{x.get()}",
        true,
    ),
    (
        "redundant-equality",
        "trait P{type A;fn get(&self)->Self::A;}trait T:P<A=u8>{}fn f(x:&dyn T<A=u8>)->u8{x.get()}",
        true,
    ),
    (
        "membership",
        "trait T{}fn need<X:T+?Sized>(_:&X){}fn f(x:&dyn T){need(x)}",
        true,
    ),
    (
        "unrelated",
        "trait T{}trait U{}fn need<X:U+?Sized>(_:&X){}fn f(x:&dyn T){need(x)}",
        false,
    ),
    (
        "supertrait",
        "trait P{}trait T:P{}fn need<X:P+?Sized>(_:&X){}fn f(x:&dyn T){need(x)}",
        true,
    ),
    (
        "trait-input",
        "trait T<A>{}fn need<X:T<u8>+?Sized>(_:&X){}fn f(x:&dyn T<u8>){need(x)}",
        true,
    ),
    (
        "wrong-input",
        "trait T<A>{}fn need<X:T<u8>+?Sized>(_:&X){}fn f(x:&dyn T<u16>){need(x)}",
        false,
    ),
    (
        "associated",
        "trait T{type A;}fn need<X:T<A=u8>+?Sized>(_:&X){}fn f(x:&dyn T<A=u8>){need(x)}",
        true,
    ),
    (
        "wrong-associated",
        "trait T{type A;}fn need<X:T<A=u16>+?Sized>(_:&X){}fn f(x:&dyn T<A=u8>){need(x)}",
        false,
    ),
    (
        "inherited-equality",
        "trait P{type A;}trait T:P<A=u8>{}fn need<X:P<A=u8>+?Sized>(_:&X){}fn f(x:&dyn T){need(x)}",
        true,
    ),
    (
        "inherited-input",
        "trait P<A>{}trait T:P<u8>{}fn need<X:P<u8>+?Sized>(_:&X){}fn f(x:&dyn T){need(x)}",
        true,
    ),
    (
        "shared-call",
        "trait T{fn get(&self)->u8;}fn f(x:&dyn T)->u8{x.get()}",
        true,
    ),
    (
        "mutable-call",
        "trait T{fn set(&mut self,x:u8);}fn f(x:&mut dyn T){x.set(4)}",
        true,
    ),
    (
        "shared-mutation",
        "trait T{fn set(&mut self,x:u8);}fn f(x:&dyn T){x.set(4)}",
        false,
    ),
    (
        "result",
        "trait T{type A;fn get(&self)->Self::A;}fn f(x:&dyn T<A=u8>)->u8{x.get()}",
        true,
    ),
    (
        "wrong-result",
        "trait T{type A;fn get(&self)->Self::A;}fn f(x:&dyn T<A=u8>)->u16{x.get()}",
        false,
    ),
    (
        "argument",
        "trait T{type A;fn set(&self,x:Self::A);}fn f(x:&dyn T<A=u8>){x.set(7u8)}",
        true,
    ),
    (
        "wrong-argument",
        "trait T{type A;fn set(&self,x:Self::A);}fn f(x:&dyn T<A=u8>){x.set(7u16)}",
        false,
    ),
    (
        "hidden-trait",
        "mod m{pub trait T{fn get(&self)->u8;}}fn f(x:&dyn m::T)->u8{x.get()}",
        true,
    ),
    (
        "hidden-supertrait",
        "mod m{pub trait P{fn get(&self)->u8;}pub trait T:P{}}fn f(x:&dyn m::T)->u8{x.get()}",
        true,
    ),
    (
        "supertrait-result",
        "trait P{type A;fn get(&self)->Self::A;}trait T:P<A=u8>{}fn f(x:&dyn T)->u8{x.get()}",
        true,
    ),
    (
        "qualified",
        "trait T{fn get(&self)->u8;}fn f(x:&dyn T)->u8{T::get(x)}",
        true,
    ),
    (
        "qualified-type",
        "trait T{fn get(&self)->u8;}fn f(x:&dyn T)->u8{<dyn T>::get(x)}",
        true,
    ),
    (
        "sized-optout",
        "trait T{fn get(&self)->u8 where Self:Sized;}fn f(x:&dyn T)->u8{x.get()}",
        false,
    ),
    (
        "generic-optout",
        "trait T{fn get<A>(&self,x:A) where Self:Sized;}fn f(x:&dyn T){x.get(7u8)}",
        false,
    ),
    (
        "quantified-member",
        "trait T<'a>{fn get(&self,x:&'a u8)->u8;}fn f(x:&dyn for<'a>T<'a>, y:&u8)->u8{x.get(y)}",
        true,
    ),
    (
        "quantified-super",
        "trait P<'a>{fn get(&self,x:&'a u8)->u8;}trait T:for<'a>P<'a>{}fn f(x:&dyn T,y:&u8)->u8{x.get(y)}",
        true,
    ),
    (
        "quantified-equation",
        "trait T<'a>{type A;fn get(&self,x:&'a u8)->Self::A;}fn f<'a>(x:&dyn for<'b>T<'b,A=&'b u8>,y:&'a u8)->&'a u8{x.get(y)}",
        true,
    ),
];

#[test]
fn object_trait_and_member_outcomes_match_rustc() -> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, source, accepted) in CASES {
        rustc::qualify(&format!("object-member-{name}"), source, accepted)?;
        let result = check_body_proofs(source);
        if result.is_ok() != accepted {
            failures.push(format!("{name}: {:?}", result.err()));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

fn check_body_proofs(source: &str) -> Result<(), String> {
    use rustgm::{body, borrow, types::copy::Copies};
    let (report, mut facts) = semantic::analyze(source)?;
    let mut program = body::build_concrete(&report, &facts).map_err(|e| e.to_string())?;
    let mut templates = body::templates::build(&report, &facts).map_err(|e| e.to_string())?;
    body::promote::analyze(&report, &mut facts, &mut program).map_err(|e| e.to_string())?;
    body::promote::analyze_templates(&report, &mut facts, &mut templates)
        .map_err(|e| e.to_string())?;
    let mut contracts = body::regions::collect(&facts).map_err(str::to_owned)?;
    let mut copies = Copies::default();
    let mut environment = borrow::Environment {
        types: &facts.types,
        definitions: &facts.aggregates,
        derives: &facts.derives,
        copies: &mut copies,
    };
    borrow::templates::verify(
        &mut environment,
        &facts.lifetimes.variances,
        &contracts,
        &templates,
        &[],
        |_, _| Err("object member controls have no owned nominal fields"),
    )
    .map_err(|e| e.to_string())?;
    let sources: std::collections::HashSet<_> =
        program.bodies.iter().map(|body| body.source).collect();
    contracts
        .bodies
        .retain(|body| sources.contains(&body.source));
    let regions = borrow::regions::check_bodies(
        &facts.types,
        &facts.lifetimes.variances,
        &contracts,
        &program,
    )
    .map_err(|e| e.to_string())?;
    borrow::verify(&mut environment, &regions, &program)
        .map(|_| ())
        .map_err(|e| e.to_string())
}
