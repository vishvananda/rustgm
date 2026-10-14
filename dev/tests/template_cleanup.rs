#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;
use rustgm::{
    body, borrow,
    types::{Kind, copy::Copies},
};

const CASES: &[(&str, &str, bool)] = &[
    (
        "move-live",
        "fn f<T>(x:T){let r=&x;let y=x;let z=r;}",
        false,
    ),
    (
        "move-ended",
        "fn f<T>(x:T){let r=&x;let z=r;let y=x;}",
        true,
    ),
    (
        "replace-live",
        "fn f<T>(mut x:T,y:T){let r=&x;x=y;let z=r;}",
        false,
    ),
    (
        "replace-ended",
        "fn f<T>(mut x:T,y:T){let r=&x;let z=r;x=y;}",
        true,
    ),
    ("return-transfer", "fn f<T>(x:T)->T{x}", true),
    (
        "branch-cleanup",
        "fn f<T>(x:T,b:bool){if b{let y=x;}}",
        true,
    ),
    (
        "branch-borrow-live",
        "fn f<T>(x:T,b:bool){let r=&x;if b{let y=x;}let z=r;}",
        false,
    ),
    (
        "branch-borrow-ended",
        "fn f<T>(x:T,b:bool){let r=&x;let z=r;if b{let y=x;}}",
        true,
    ),
    (
        "drop-live",
        "struct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){}}fn f<T>(x:T){let mut n=0u8;let d=D(&mut n);n=1;}",
        false,
    ),
    (
        "drop-ended",
        "struct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){}}fn f<T>(x:T){let mut n=0u8;{let d=D(&mut n);}n=1;}",
        true,
    ),
    (
        "tuple-bound-drop",
        "struct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){}}fn f<T>(x:T){let mut n=0u8;let(d,t)=(D(&mut n),x);let y=t;n=1;}",
        false,
    ),
    (
        "tuple-wildcard-drop",
        "struct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){}}fn f<T>(x:T){let mut n=0u8;let(_,t)=(D(&mut n),x);n=1;let y=t;}",
        true,
    ),
    (
        "tuple-partial-ended",
        "struct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){}}fn f<T>(x:T){let mut n=0u8;let pair=(D(&mut n),x);{let d=pair.0;}n=1;let t=pair.1;}",
        false,
    ),
    (
        "tuple-partial-live",
        "struct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){}}fn f<T>(x:T){let mut n=0u8;let pair=(D(&mut n),x);let t=pair.1;n=1;}",
        false,
    ),
    (
        "zero-repeat-operand",
        "struct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){}}fn f<T>(x:T){let mut n=0u8;let a=[D(&mut n);0];n=1;}",
        true,
    ),
    (
        "one-repeat-discard",
        "struct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){}}fn f<T>(x:T){let mut n=0u8;[D(&mut n);1];n=1;}",
        true,
    ),
    (
        "replace-different-source",
        "struct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){}}fn f<T>(x:T){let mut n=0u8;let mut m=0u8;let mut d=D(&mut n);let new=D(&mut m);d=new;n=1;}",
        true,
    ),
    (
        "replace-same-source",
        "struct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){}}fn f<T>(x:T){let mut n=0u8;let mut d=D(&mut n);d=D(&mut n);}",
        false,
    ),
    (
        "const-repeat-owned",
        "struct D;impl Drop for D{fn drop(&mut self){}}const C:D=D;fn f<T,const N:usize>(x:T)->[D;N]{[C;N]}",
        true,
    ),
    (
        "ordinary-repeat-owned-body",
        "fn f<T:Copy,U,const N:usize>(x:T,u:U)->[T;N]{[x;N]}",
        true,
    ),
    (
        "ordinary-repeat-noncopy",
        "fn f<T,U,const N:usize>(x:T,u:U)->[T;N]{[x;N]}",
        false,
    ),
    (
        "zero-promotion-cleanup",
        "fn f<T:'static>(x:T)->&'static[T;0]{&[x;0]}",
        true,
    ),
    (
        "tuple-partial-no-use",
        "struct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){}}fn f<T>(x:T){let mut n=0u8;let pair=(D(&mut n),x);{let d=pair.0;}n=1;}",
        false,
    ),
    (
        "tuple-copy-no-use",
        "struct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){}}fn f<T:Copy>(x:T){let mut n=0u8;let pair=(D(&mut n),x);{let d=pair.0;}n=1;}",
        false,
    ),
    (
        "tuple-fully-moved",
        "struct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){}}fn f<T>(x:T){let mut n=0u8;let pair=(D(&mut n),x);{let d=pair.0;}let t=pair.1;n=1;}",
        false,
    ),
    (
        "tuple-copy-tail-read",
        "struct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){}}fn f<T:Copy>(x:T){let mut n=0u8;let pair=(D(&mut n),x);{let d=pair.0;}let t=pair.1;n=1;}",
        false,
    ),
    (
        "tuple-pattern-ended",
        "struct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){}}fn f<T>(x:T){let mut n=0u8;let pair=(D(&mut n),x);let t;{let(d,y)=pair;t=y;}n=1;let z=t;}",
        false,
    ),
    (
        "tuple-whole-move",
        "struct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){}}fn f<T>(x:T){let mut n=0u8;let pair=(D(&mut n),x);{let other=pair;}n=1;}",
        true,
    ),
    (
        "tuple-scope-ended",
        "struct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){}}fn f<T>(x:T){let mut n=0u8;let t={let pair=(D(&mut n),x);let(d,t)=pair;t};n=1;let z=t;}",
        true,
    ),
    (
        "tuple-helper-transfer",
        "struct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){}}fn take<T>(pair:(D<'_>,T))->T{let(d,t)=pair;t}fn f<T>(x:T){let mut n=0u8;let pair=(D(&mut n),x);let t=take(pair);n=1;let z=t;}",
        true,
    ),
    (
        "tuple-reference-after-partial",
        "struct D;impl Drop for D{fn drop(&mut self){}}fn f<T>(x:T){let mut n=0u8;let r;{let w=(D,&mut n);{let d=w.0;}r=&mut*w.1;}*r=1;}",
        true,
    ),
    (
        "nominal-reference-after-partial",
        "struct D;impl Drop for D{fn drop(&mut self){}}struct W<'a>{d:D,r:&'a mut u8}fn f<T>(x:T){let mut n=0u8;let r;{let w=W{d:D,r:&mut n};{let d=w.d;}r=&mut*w.r;}*r=1;}",
        true,
    ),
    (
        "sparse-array-move",
        "fn f<T>(a:[T;1000000000]){let[first,..,last]=a;}",
        true,
    ),
    (
        "array-partial-reference",
        "fn f<T>(a:[(T,&mut u8);1]){let [(t,ref r)]=a;let use_r=r;}",
        true,
    ),
    (
        "loop-cleanup-ended",
        "struct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){}}fn f<T>(x:T,b:bool){let mut n=0u8;while b{{let d=D(&mut n);}n=1;}}",
        true,
    ),
    (
        "loop-cleanup-live",
        "struct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){}}fn f<T>(x:T,b:bool){let mut n=0u8;while b{let d=D(&mut n);n=1;}}",
        false,
    ),
    (
        "nominal-lifetimes",
        "struct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){}}struct W<'a,'b>{d:D<'b>,r:&'a mut u8}fn f<T>(x:T){let mut a=0u8;let mut b=0u8;let w=W{d:D(&mut b),r:&mut a};a=1;}",
        true,
    ),
    (
        "nominal-lifetimes-conflict",
        "struct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){}}struct W<'a,'b>{d:D<'b>,r:&'a mut u8}fn f<T>(x:T){let mut a=0u8;let mut b=0u8;let w=W{d:D(&mut b),r:&mut a};b=1;}",
        false,
    ),
    (
        "enum-partial",
        "struct D;impl Drop for D{fn drop(&mut self){}}enum E{A(D,u8),B}fn f<T>(x:T,e:E){match e{E::A(d,n)=>{let z=d;},E::B=>{}}}",
        true,
    ),
    (
        "drop-field-move",
        "struct D;impl Drop for D{fn drop(&mut self){}}struct W(D);impl Drop for W{fn drop(&mut self){}}fn f<T>(x:T,w:W){let d=w.0;}",
        false,
    ),
    ("reference-only", "fn f<T>(x:&mut T)->&T{&*x}", true),
    (
        "reference-only-conflict",
        "fn f<T>(x:&mut T){let r=&mut*x;let s=&mut*x;let z=&*r;}",
        false,
    ),
    (
        "directed-parameter-bound",
        "fn f<'a,T:'a>(mut x:T,y:T){let r=&x;let z=r;x=y;}",
        true,
    ),
    (
        "static-parameter-bound",
        "fn f<T:'static>(mut x:T,y:T){let r=&x;let z=r;x=y;}",
        true,
    ),
];

fn check(source: &str) -> Result<Vec<borrow::loans::Analysis>, Box<dyn std::error::Error>> {
    let (report, mut facts) = semantic::analyze(source)?;
    let mut templates = body::templates::build(&report, &facts)?;
    body::promote::analyze_templates(&report, &mut facts, &mut templates)?;
    let contracts = body::regions::collect(&facts)?;
    let contracts = borrow::regions::check_templates(
        &facts.types,
        &facts.lifetimes.variances,
        &contracts,
        &templates,
    )?;
    verify(&facts, &templates, &contracts)
}
fn verify(
    facts: &rustgm::sema::model::Declarations,
    templates: &body::templates::Templates,
    contracts: &borrow::regions::Analysis,
) -> Result<Vec<borrow::loans::Analysis>, Box<dyn std::error::Error>> {
    let mut copies = Copies::default();
    let mut environment = borrow::Environment {
        types: &facts.types,
        definitions: &facts.aggregates,
        derives: &facts.derives,
        copies: &mut copies,
    };
    let custom: Vec<_> = facts
        .destructors
        .implementations()
        .iter()
        .map(|d| d.item)
        .collect();
    Ok(borrow::loans::templates::verify(
        &mut environment,
        templates,
        contracts,
        &custom,
        |ty, selected| {
            let (Some(Kind::Adt(owner, _)), body::ir::Selection::Nominal(field)) =
                (facts.types.kind(ty), selected)
            else {
                return Err("invalid selected nominal field");
            };
            let variant = facts
                .members
                .variant(field)
                .ok_or("missing field variant")?;
            let variant = facts
                .members
                .variants
                .get(variant.0 as usize)
                .ok_or("missing variant")?;
            if variant.owner != owner {
                return Err("foreign selected field");
            }
            Ok((
                variant.ordinal,
                facts
                    .members
                    .ordinal(field)
                    .ok_or("missing field ordinal")?,
            ))
        },
    )?)
}

#[test]
fn declared_cleanup_outcomes_match_rustc() -> Result<(), Box<dyn std::error::Error>> {
    for (name, source, accepted) in CASES {
        rustc::qualify(&format!("template-cleanup-{name}"), source, *accepted)?;
    }
    Ok(())
}
#[test]
fn generic_cleanup_uses_initialization_and_destructor_liveness() -> Result<(), String> {
    let mut failures = Vec::new();
    for (name, source, accepted) in CASES {
        let result = check(source);
        if result.is_ok() != *accepted {
            failures.push(format!(
                "{name}: wanted {accepted}; {}",
                result.err().map_or_else(String::new, |e| e.to_string())
            ));
        } else if !accepted && let Err(error) = result {
            let expected = match *name {
                "ordinary-repeat-noncopy" => "repeated array element must implement Copy",
                "drop-field-move" => {
                    "cannot move a non-Copy field out of a type that implements Drop"
                }
                _ => "access conflicts with a live reference loan",
            };
            if !error.to_string().contains(expected) {
                failures.push(format!("{name}: expected {expected}; {error}"));
            }
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n"))
    }
}

#[test]
fn cleanup_verification_requires_complete_matching_contracts()
-> Result<(), Box<dyn std::error::Error>> {
    let (report, facts) = semantic::analyze("fn f<T>(x:T){}fn g<U>(x:U){}")?;
    let mut templates = body::templates::build(&report, &facts)?;
    let collected = body::regions::collect(&facts)?;
    let provenance = || {
        borrow::regions::check_templates(
            &facts.types,
            &facts.lifetimes.variances,
            &collected,
            &templates,
        )
    };
    let mut contracts = provenance()?;
    verify(&facts, &templates, &contracts)?;
    contracts.bodies[1].source = contracts.bodies[0].source;
    assert!(verify(&facts, &templates, &contracts).is_err());
    let mut contracts = provenance()?;
    contracts.bodies.pop();
    assert!(verify(&facts, &templates, &contracts).is_err());
    let contracts = provenance()?;
    templates.bodies[0].destruction.values.clear();
    assert!(verify(&facts, &templates, &contracts).is_err());
    Ok(())
}
