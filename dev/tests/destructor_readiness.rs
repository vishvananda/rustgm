//! Early declaration constants require complete nominal facts and exact destructor owners.
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;
use std::fmt::Write as _;

#[test]
fn deep_forward_aliases_reserve_one_actual_drop_owner() -> Result<(), Box<dyn std::error::Error>> {
    let mut source = String::from("struct S{n:usize}static V:S=S{n:3};");
    for index in 0..1024 {
        write!(source, "type A{index}<T>=A{}<T>;", index + 1)?;
    }
    source.push_str("type A1024<T>=T;type D<T=A0<S>,const N:usize={V.n}>=T;impl Drop for D{fn drop(&mut self){}}");
    let (report, facts) = semantic::analyze(&source)?;
    let [implementation] = facts.destructors.implementations() else {
        return Err("expected one destructor".into());
    };
    let definition = &report.resolution.definitions[implementation.item.0 as usize];
    assert_eq!(report.package.names.get(definition.name), "S");
    Ok(())
}

#[test]
fn accepted_dependencies_complete_before_evaluation() -> Result<(), String> {
    for &(name, accepted, source) in CASES {
        if accepted {
            semantic::analyze(source).map_err(|e| format!("{name}: {e}"))?;
        }
    }
    Ok(())
}
#[test]
fn invalid_dependencies_remain_rejections() {
    for &(name, accepted, source) in CASES {
        if !accepted {
            assert!(semantic::analyze(source).is_err(), "{name}: {source}");
        }
    }
}
#[test]
fn declaration_outcomes_match_independent_rustc() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, accepted, source) in CASES {
        rustc::qualify(&format!("drop-owner-readiness-{name}"), source, accepted)?;
    }
    Ok(())
}

const CASES: &[(&str, bool, &str)] = &[
    (
        "own-owned-projection",
        false,
        "struct S{n:usize}type A<const N:usize>=S;impl Drop for A<{S{n:3}.n}>{fn drop(&mut self){}}",
    ),
    (
        "own-literal",
        true,
        "struct S{n:usize}type A<const N:usize>=S;impl Drop for A<3>{fn drop(&mut self){}}",
    ),
    (
        "other-later-drop",
        false,
        "struct S{n:usize}struct T{n:usize}type A<const N:usize>=S;impl Drop for A<{T{n:3}.n}>{fn drop(&mut self){}}impl Drop for T{fn drop(&mut self){}}",
    ),
    (
        "other-earlier-drop",
        false,
        "struct S{n:usize}struct T{n:usize}type A<const N:usize>=S;impl Drop for T{fn drop(&mut self){}}impl Drop for A<{T{n:3}.n}>{fn drop(&mut self){}}",
    ),
    (
        "other-no-drop",
        true,
        "struct S{n:usize}struct T{n:usize}type A<const N:usize>=S;impl Drop for A<{T{n:3}.n}>{fn drop(&mut self){}}",
    ),
    (
        "own-static-read",
        true,
        "struct S{n:usize}static V:S=S{n:3};type A<const N:usize>=S;impl Drop for A<{V.n}>{fn drop(&mut self){}}",
    ),
    (
        "own-const-borrow-read",
        true,
        "struct S{n:usize}const R:&S=&S{n:3};type A<const N:usize>=S;impl Drop for A<{R.n}>{fn drop(&mut self){}}",
    ),
    (
        "own-unused-default-projection",
        false,
        "struct S{n:usize}type A<const N:usize={S{n:3}.n}>=S;impl Drop for A<3>{fn drop(&mut self){}}",
    ),
    (
        "own-unused-default-static",
        true,
        "struct S{n:usize}static V:S=S{n:3};type A<const N:usize={V.n}>=S;impl Drop for A<3>{fn drop(&mut self){}}",
    ),
    (
        "parameter-alias",
        true,
        "struct S{n:usize}type A<T>=T;impl Drop for A<S>{fn drop(&mut self){}}",
    ),
    (
        "default-parameter-alias",
        true,
        "struct S{n:usize}type A<T=S>=T;impl Drop for A{fn drop(&mut self){}}",
    ),
    (
        "nested-parameter-alias",
        true,
        "struct S{n:usize}type Id<T>=T;type A<T=Id<S>>=Id<T>;impl Drop for A{fn drop(&mut self){}}",
    ),
    (
        "finite-self-default",
        true,
        "struct S{n:usize}type A<T=A<S>>=T;impl Drop for A{fn drop(&mut self){}}",
    ),
    (
        "finite-mutual-default",
        true,
        "struct S{n:usize}type A<T=B<S>>=T;type B<T=A<S>>=T;impl Drop for A{fn drop(&mut self){}}",
    ),
    (
        "parameter-owned-argument",
        false,
        "struct S{n:usize}type A<T,const N:usize>=T;impl Drop for A<S,{S{n:3}.n}>{fn drop(&mut self){}}",
    ),
    (
        "parameter-static-argument",
        true,
        "struct S{n:usize}static V:S=S{n:3};type A<T,const N:usize>=T;impl Drop for A<S,{V.n}>{fn drop(&mut self){}}",
    ),
    (
        "parameter-default-owned",
        false,
        "struct S{n:usize}type A<T=S,const N:usize={S{n:3}.n}>=T;impl Drop for A{fn drop(&mut self){}}",
    ),
    (
        "parameter-default-static",
        true,
        "struct S{n:usize}static V:S=S{n:3};type A<T=S,const N:usize={V.n}>=T;impl Drop for A{fn drop(&mut self){}}",
    ),
    (
        "later-owner-alias",
        false,
        "struct S{n:usize}struct T{n:usize}type A<X,const N:usize>=X;type B<X=T>=X;impl Drop for A<S,{T{n:3}.n}>{fn drop(&mut self){}}impl Drop for B{fn drop(&mut self){}}",
    ),
    (
        "no-other-owner",
        true,
        "struct S{n:usize}struct T{n:usize}type A<X,const N:usize>=X;impl Drop for A<S,{T{n:3}.n}>{fn drop(&mut self){}}",
    ),
    (
        "nonnominal-target",
        false,
        "struct S{n:usize}type A<T>=T;impl Drop for A<[S;1]>{fn drop(&mut self){}}",
    ),
    (
        "invalid-unused-parameter",
        false,
        "struct S{n:usize}type A<T>=S;impl Drop for A<u8>{fn drop(&mut self){}}",
    ),
    (
        "real-default-cycle",
        false,
        "struct S{n:usize}type A<T=A>=T;impl Drop for A{fn drop(&mut self){}}",
    ),
    (
        "alias-cycle",
        false,
        "struct S{n:usize}type A=B;type B=A;impl Drop for A{fn drop(&mut self){}}",
    ),
    (
        "early-enum-drop-header",
        false,
        "enum E{A=3}type A<const N:usize>=E;impl Drop for A<{E::A as usize}>{fn drop(&mut self){}}",
    ),
    (
        "drop-head-unused-const-default",
        false,
        "struct S{n:usize}type A<T=A<S>,const N:usize={S{n:3}.n}>=T;impl Drop for A<S,3>{fn drop(&mut self){}}",
    ),
    (
        "drop-head-finite-self-const",
        true,
        "struct S{n:usize}static V:S=S{n:3};type A<T=A<S,3>,const N:usize={V.n}>=T;impl Drop for A{fn drop(&mut self){}}",
    ),
    (
        "drop-head-no-overapprox",
        true,
        "struct S{n:usize}struct T{n:usize}type Id<X>=X;type A<X=Id<T>,const N:usize={S{n:3}.n}>=X;impl Drop for A{fn drop(&mut self){}}",
    ),
    (
        "drop-head-lifetime",
        false,
        "struct S<'a>{r:&'a usize}type A<'a,T=S<'a>,const N:usize=3>=T;impl<'a> Drop for A<'a,S<'a>,{*S{r:&3}.r}>{fn drop(&mut self){}}",
    ),
];
