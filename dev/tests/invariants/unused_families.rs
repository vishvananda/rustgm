//! Unused families preserve the trait's application domain and prove their result independently.
use super::{Checker, TestResult, report, reserve};
use crate::{resolve::model::DefKind, sema::associated::OwnerKind};
#[path = "local_family_contracts.rs"]
mod local_contracts;

const CASES: &[(&str, &str, bool)] = &[
    (
        "result-type-argument",
        "trait A<T>{}impl<T>A<T> for T{}trait Q{type Out<T>:A<T>;}struct S;impl Q for S{type Out<U>=U;}",
        true,
    ),
    (
        "wrong-result-argument",
        "trait A<T>{}impl<T>A<T> for T{}trait Q{type Out<T,U>:A<T>;}struct S;impl Q for S{type Out<X,Y>=Y;}",
        false,
    ),
    (
        "const-result-argument",
        "trait A<const N:usize>{}impl<const N:usize>A<N> for [u8;N]{}trait Q{type Out<const N:usize>:A<N>;}struct S;impl Q for S{type Out<const M:usize>=[u8;M];}",
        true,
    ),
    (
        "quantified-result",
        "trait A<'a>{}trait Q{type Out<T:for<'a>A<'a>>:for<'a>A<'a>;}struct S;impl Q for S{type Out<U:for<'b>A<'b>>=U;}",
        true,
    ),
    (
        "zero-parameters",
        "trait Q{type Out where Self:Copy;}impl Q for u8{type Out=u8;}",
        true,
    ),
    (
        "zero-parameters-unproved",
        "trait Q{type Out where Self:Copy;}struct S;impl Q for S{type Out=u8;}",
        false,
    ),
    (
        "identity",
        "trait Q{type Out<T>;}struct S;impl Q for S{type Out<T>=T;}",
        true,
    ),
    (
        "const-array",
        "trait Q{type Out<const N:usize>;}struct S;impl Q for S{type Out<const M:usize>=[u8;M];}",
        true,
    ),
    (
        "wrong-const-kind",
        "trait Q{type Out<const N:usize>;}struct S;impl Q for S{type Out<const M:u8>=u8;}",
        false,
    ),
    (
        "wrong-arity",
        "trait Q{type Out<T,U>;}struct S;impl Q for S{type Out<T>=T;}",
        false,
    ),
    (
        "wrong-kind",
        "trait Q{type Out<T>;}struct S;impl Q for S{type Out<const N:usize>=u8;}",
        false,
    ),
    (
        "stronger-copy",
        "trait Q{type Out<T>;}struct S;impl Q for S{type Out<T:Copy>=T;}",
        false,
    ),
    (
        "declared-copy",
        "trait Q{type Out<T:Copy>:Copy;}struct S;impl Q for S{type Out<T:Copy>=T;}",
        true,
    ),
    (
        "omitted-copy",
        "trait Q{type Out<T:Copy>:Copy;}struct S;impl Q for S{type Out<T>=T;}",
        false,
    ),
    (
        "unproved-result",
        "trait Q{type Out<T>:Copy;}struct S;impl Q for S{type Out<T>=T;}",
        false,
    ),
    (
        "stronger-sized",
        "trait Q{type Out<T:?Sized>;}struct S;impl Q for S{type Out<T>=u8;}",
        false,
    ),
    (
        "weaker-sized",
        "trait Q{type Out<T>;}struct S;impl Q for S{type Out<T:?Sized>=u8;}",
        false,
    ),
    (
        "implied-sized",
        "trait Q{type Out<T:?Sized+Copy>;}struct S;impl Q for S{type Out<T>=u8;}",
        false,
    ),
    (
        "stronger-user",
        "trait Mark{}trait Q{type Out<T>;}struct S;impl Q for S{type Out<T:Mark>=T;}",
        false,
    ),
    (
        "inherited-user",
        "trait A{}trait B:A{}trait Q{type Out<T:B>:A;}struct S;impl Q for S{type Out<T>=T;}",
        false,
    ),
    (
        "lifetime-input",
        "trait Q<T>{type Out<'a> where T:'a;}struct S;impl<T>Q<T>for S{type Out<'a>=&'a T where T:'a;}",
        true,
    ),
    (
        "omitted-lifetime-input",
        "trait Q<T>{type Out<'a> where T:'a;}struct S;impl<T>Q<T>for S{type Out<'a>=&'a T;}",
        false,
    ),
    (
        "stronger-lifetime",
        "trait Q{type Out<'a,'b>;}struct S;impl Q for S{type Out<'a,'b>=u8 where 'a:'b;}",
        false,
    ),
    (
        "declared-lifetime",
        "trait Q{type Out<'a,'b> where 'a:'b;}struct S;impl Q for S{type Out<'a,'b>=&'b &'a u8;}",
        false,
    ),
    (
        "unproved-lifetime-result",
        "trait Q{type Out<'a,'b>;}struct S;impl Q for S{type Out<'a,'b>=&'b &'a u8;}",
        false,
    ),
    (
        "parent-capture",
        "trait Q<'p,T>{type Out<'a,U> where T:'p,U:'a;}struct S;impl<'p,T>Q<'p,T>for S{type Out<'a,U>=(&'p T,&'a U);}",
        false,
    ),
    (
        "repeated-inherited",
        "trait A{}trait B:A{}trait Q{type Out<T:B>:A;}struct S;impl Q for S{type Out<T:B>=T;}",
        true,
    ),
    (
        "repeated-lifetimes",
        "trait Q{type Out<'a,'b> where 'a:'b;}struct S;impl Q for S{type Out<'x,'y>=&'y &'x u8 where 'x:'y;}",
        true,
    ),
    (
        "repeated-captures",
        "trait Q<'p,T>{type Out<'a,U> where T:'p,U:'a;}struct S;impl<'p,T>Q<'p,T>for S{type Out<'a,U>=(&'p T,&'a U) where T:'p,U:'a;}",
        true,
    ),
    (
        "copy-implies-sized",
        "trait Q{type Out<T:?Sized+Copy>;}struct S;impl Q for S{type Out<T:Copy>=u8;}",
        true,
    ),
    (
        "erased-copy",
        "trait Q{type Out<T:Copy>;}struct S;impl Q for S{type Out<T>=u8;}",
        false,
    ),
    (
        "erased-lifetime",
        "trait Q{type Out<'a,'b> where 'a:'b;}struct S;impl Q for S{type Out<'a,'b>=u8;}",
        false,
    ),
    (
        "parent-bound",
        "trait Q{type Out<'a,T> where T:'a;}struct S;impl Q for S{type Out<'x,U:'x>=&'x U;}",
        true,
    ),
    (
        "unconstrained-capture",
        "trait Q{type Out<'a>;}struct S;impl<'p>Q for S{type Out<'a>=&'p u8;}",
        false,
    ),
    (
        "missing-family",
        "trait Q{type Out<T>;}struct S;impl Q for S{type Out=u8;}",
        false,
    ),
    (
        "extra-family",
        "trait Q{type Out;}struct S;impl Q for S{type Out<T>=u8;}",
        false,
    ),
    (
        "relaxed-both",
        "trait Q{type Out<T:?Sized>;}struct S;impl Q for S{type Out<T:?Sized>=u8;}",
        true,
    ),
    (
        "quantified",
        "trait A<'a>{}trait Q{type Out<T> where for<'a>T:A<'a>;}struct S;impl Q for S{type Out<U>=U where for<'x>U:A<'x>;}",
        true,
    ),
    (
        "stronger-universal",
        "trait A<'a>{}trait Q{type Out<T:A<'static>>;}struct S;impl Q for S{type Out<U>=U where for<'x>U:A<'x>;}",
        false,
    ),
    (
        "result-lifetime",
        "trait Q{type Out<'a>:'a;}struct S;impl Q for S{type Out<'a>=&'a u8;}",
        true,
    ),
    (
        "wrong-result-lifetime",
        "trait Q{type Out<'a,'b>:'a;}struct S;impl Q for S{type Out<'a,'b>=&'b u8;}",
        false,
    ),
    (
        "two-independent",
        "trait Q{type Out<T:Copy,U>;}struct S;impl Q for S{type Out<T,U:Copy>=u8;}",
        false,
    ),
];

fn check(source: &str) -> TestResult {
    let report = report(source)?;
    let mut checker = Checker::new(&report)?;
    prepare(&mut checker)?;
    let headers: Vec<_> = checker
        .facts
        .generics
        .implementations
        .as_ref()
        .ok_or("missing impls")?
        .headers()
        .map(|h| (h.owner, h.source, h.predicate))
        .collect();
    let members =
        crate::sema::declarations::traits::associated_types::Members::new(&report, &checker.facts);
    for (owner, source, predicate) in headers {
        checker.trait_impl_types(owner, source, predicate, &members)?;
    }
    Ok(())
}

pub(super) fn prepare(checker: &mut Checker<'_>) -> TestResult {
    let report = checker.report;
    reserve(checker)?;
    let members = checker.facts.associated.members().to_vec();
    // Exercise the production conformance entry point after preparing typed
    // family declarations. This does not bypass the public source gate in tests
    // that claim source support; that gate remains closed during integration.
    for member in &members {
        if report.resolution.definitions[member.definition.0 as usize].kind != DefKind::TypeAlias {
            continue;
        }
        let owner = checker.facts.associated.owners()[member.owner.0 as usize];
        let key = report.resolution.definitions[member.definition.0 as usize]
            .node
            .ok_or("missing source")?;
        if let OwnerKind::Trait { definition, .. } = owner.kind {
            checker.declare_associated_type(member.definition, key, owner, definition)?;
            checker.complete_associated_result_bounds(member.definition, key, &mut 0, 100_000)?;
        }
    }
    if let Some(contracts) = &mut checker.facts.generics.traits {
        contracts.complete_results();
    }
    crate::sema::traits::results::prepare_outlives(&mut checker.facts)?;
    for member in &members {
        let owner = checker.facts.associated.owners()[member.owner.0 as usize];
        if report.resolution.definitions[member.definition.0 as usize].kind == DefKind::TypeAlias
            && matches!(owner.kind, OwnerKind::Impl { .. })
        {
            checker
                .facts
                .generics
                .parents
                .insert(member.definition, owner.self_definition);
            checker.definition(member.definition, false)?;
            checker.drain(&mut 0, 100_000)?;
        }
    }
    checker.complete_deferred_declarations(&mut 0, 100_000)?;
    checker.complete_nominals(&mut 0, 100_000)?;
    Ok(())
}

#[test]
fn unused_family_definitions_match_independently_qualified_contracts() {
    let mut failures = Vec::new();
    for &(name, source, accepted) in CASES {
        let result = check(source);
        if result.is_ok() != accepted {
            failures.push(format!(
                "{name}: expected accepted={accepted}, result={result:?}"
            ));
        } else if let Err(error) = result {
            let message = error.to_string();
            assert!(
                [
                    "associated family generic parameter",
                    "associated family const parameter type mismatch",
                    "associated family definition does not satisfy",
                    "associated family definition must be well-formed",
                    "associated family lifetime contract is not satisfied",
                    "associated type uses an impl lifetime",
                ]
                .iter()
                .any(|class| message.contains(class)),
                "{name}: {message}"
            );
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
