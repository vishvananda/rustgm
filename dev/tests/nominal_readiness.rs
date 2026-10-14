//! Early declaration constants require complete nominal facts and exact destructor owners.
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;
use std::fmt::Write as _;

#[test]
fn repeated_defaults_share_fields_and_keep_the_selected_length()
-> Result<(), Box<dyn std::error::Error>> {
    let mut source = String::from("struct S(usize);fn expected(_: [u8;3]){}");
    for index in 0..512 {
        write!(
            source,
            "type A{index}<const N:usize={{S(3).0}}>=[u8;N];fn f{index}(_:A{index}){{}}"
        )?;
    }
    let (_, facts) = semantic::analyze(&source)?;
    assert_eq!((&facts.aggregates).into_iter().count(), 1);
    let expected = facts
        .functions
        .first()
        .ok_or("missing control function")?
        .signature;
    assert_eq!(facts.functions.len(), 513);
    assert!(facts.functions.iter().all(|f| f.signature == expected));
    Ok(())
}

#[test]
fn field_queries_distinguish_finite_defaults_from_storage_dependencies() -> Result<(), String> {
    semantic::analyze(
        "struct S{a:A<3>}type A<const N:usize={S{a:[3;3]}.a[0] as usize}>=[u8;N];fn f(){let _:A=[0;3];}",
    )?;
    let source = "struct S{a:A}type A<const N:usize={S{a:[]}.a[0] as usize}>=[u8;N];";
    let error = semantic::analyze(source)
        .err()
        .ok_or("accepted cyclic field/default dependency")?;
    assert!(
        error.contains("cycle") || error.contains("recursive"),
        "{error}"
    );
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
        rustc::qualify(&format!("nominal-readiness-{name}"), source, accepted)?;
    }
    Ok(())
}

const CASES: &[(&str, bool, &str)] = &[
    (
        "nominal-default",
        true,
        "struct S{n:usize}type A<const N:usize={S{n:3}.n}>=[u8;N];fn f(_:A){}",
    ),
    (
        "nominal-named-default",
        true,
        "struct S{n:usize}const C:usize=S{n:3}.n;type A<const N:usize=C>=[u8;N];fn f(_:A){}",
    ),
    (
        "nominal-signature-argument",
        true,
        "struct S{n:usize}type A<const N:usize>=[u8;N];fn f(_:A<{S{n:3}.n}>){}",
    ),
    (
        "tuple-default",
        true,
        "type A<const N:usize={(3usize,4usize).0}>=[u8;N];fn f(_:A){}",
    ),
    (
        "array-default",
        true,
        "type A<const N:usize={[1usize,2usize,3usize][2]}>=[u8;N];fn f(_:A){}",
    ),
    (
        "tuple-constructor",
        true,
        "struct S(usize);type A<const N:usize={S(3).0}>=[u8;N];fn f(_:A){}",
    ),
    (
        "record-alias",
        true,
        "struct S{n:usize}type B=S;type A<const N:usize={B{n:3}.n}>=[u8;N];fn f(_:A){}",
    ),
    (
        "nested-record",
        true,
        "struct Inner{n:usize}struct Outer{v:Inner}type A<const N:usize={Outer{v:Inner{n:3}}.v.n}>=[u8;N];fn f(_:A){}",
    ),
    (
        "named-nominal-const",
        true,
        "struct S{n:usize}const C:S=S{n:3};type A<const N:usize={C.n}>=[u8;N];fn f(_:A){}",
    ),
    (
        "nominal-static-read",
        true,
        "struct S{n:usize}static C:S=S{n:3};type A<const N:usize={C.n}>=[u8;N];fn f(_:A){}",
    ),
    (
        "nominal-static-borrow",
        true,
        "struct S{n:usize}static C:S=S{n:3};type A<const N:usize={(&C).n}>=[u8;N];fn f(_:A){}",
    ),
    (
        "reference-projection",
        true,
        "struct S{n:usize}type A<const N:usize={(&S{n:3}).n}>=[u8;N];fn f(_:A){}",
    ),
    (
        "packed-owned-projection",
        true,
        "#[repr(packed)]struct S{n:usize}type A<const N:usize={S{n:3}.n}>=[u8;N];fn f(_:A){}",
    ),
    (
        "packed-borrow",
        false,
        "#[repr(packed)]struct S{n:usize}type A<const N:usize={*(&S{n:3}.n)}>=[u8;N];fn f(_:A){}",
    ),
    (
        "private-field",
        false,
        "mod m{pub struct S{n:usize}}type A<const N:usize={m::S{n:3}.n}>=[u8;N];fn f(_:A){}",
    ),
    (
        "drop-temporary",
        false,
        "struct S{n:usize}impl Drop for S{fn drop(&mut self){}}type A<const N:usize={S{n:3}.n}>=[u8;N];fn f(_:A){}",
    ),
    (
        "local-enum-correct",
        true,
        "enum E{A=7,B}type A<const N:usize>=[u8;N];fn f(){let _:A<{E::B as usize}>=[0;8];}",
    ),
    (
        "local-enum-wrong",
        false,
        "enum E{A=7,B}type A<const N:usize>=[u8;N];fn f(){let _:A<{E::B as usize}>=[0;0];}",
    ),
    (
        "default-enum",
        true,
        "enum E{A=7,B}type A<const N:usize={E::B as usize}>=[u8;N];fn f(_:A){}",
    ),
    (
        "default-enum-named",
        true,
        "enum E{A=C as isize,B}const C:usize=7;type A<const N:usize={E::B as usize}>=[u8;N];fn f(_:A){}",
    ),
    (
        "enum-value-cycle",
        false,
        "enum E{A=C as isize}const C:usize=E::A as usize;type A<const N:usize=C>=[u8;N];fn f(_:A){}",
    ),
    (
        "field-template-alias",
        true,
        "struct S{n:B}type B=usize;type A<const N:usize={S{n:3}.n}>=[u8;N];fn f(_:A){}",
    ),
    (
        "field-template-default",
        true,
        "struct S{n:B}type B<T=usize>=T;type A<const N:usize={S{n:3}.n}>=[u8;N];fn f(_:A){}",
    ),
    (
        "dead-branch-private",
        false,
        "mod m{pub struct S{n:usize}}type A<const N:usize={if true{3}else{m::S{n:3}.n}}>=[u8;N];fn f(_:A){}",
    ),
    (
        "local-drop-temporary",
        false,
        "struct S{n:usize}impl Drop for S{fn drop(&mut self){}}type A<const N:usize>=[u8;N];fn f(){let _:A<{S{n:3}.n}>=[0;3];}",
    ),
    (
        "local-drop-const",
        false,
        "struct S{n:usize}impl Drop for S{fn drop(&mut self){}}const C:S=S{n:3};type A<const N:usize>=[u8;N];fn f(){let _:A<{C.n}>=[0;3];}",
    ),
    (
        "local-drop-static",
        true,
        "struct S{n:usize}impl Drop for S{fn drop(&mut self){}}static C:S=S{n:3};type A<const N:usize>=[u8;N];fn f(){let _:A<{C.n}>=[0;3];}",
    ),
    (
        "local-packed-borrow",
        false,
        "#[repr(packed)]struct S{n:usize}type A<const N:usize>=[u8;N];fn f(){let _:A<{*(&S{n:3}.n)}>=[0;3];}",
    ),
    (
        "local-packed-owned",
        true,
        "#[repr(packed)]struct S{n:usize}type A<const N:usize>=[u8;N];fn f(){let _:A<{S{n:3}.n}>=[0;3];}",
    ),
    (
        "local-enum-implicit-correct",
        true,
        "enum E{A,B,C}type A<const N:usize>=[u8;N];fn f(){let _:A<{E::C as usize}>=[0;2];}",
    ),
    (
        "local-enum-implicit-wrong",
        false,
        "enum E{A,B,C}type A<const N:usize>=[u8;N];fn f(){let _:A<{E::C as usize}>=[0;0];}",
    ),
    (
        "lifetime-record-default",
        true,
        "struct S<'a>{r:&'a usize}type A<const N:usize={*S{r:&3}.r}>=[u8;N];fn f(){let _:A=[0;3];}",
    ),
    (
        "lifetime-tuple-default",
        true,
        "struct S<'a>(&'a usize);type A<const N:usize={*S(&3).0}>=[u8;N];fn f(){let _:A=[0;3];}",
    ),
    (
        "lifetime-static-default",
        true,
        "struct S<'a>{r:&'a usize}static V:S<'static>=S{r:&3};type A<const N:usize={*V.r}>=[u8;N];fn f(){let _:A=[0;3];}",
    ),
    (
        "early-enum-default",
        true,
        "enum E{A=7,B}type A<const N:usize={E::B as usize}>=[u8;N];fn f(){let _:A=[0;8];}",
    ),
    (
        "early-enum-forward",
        true,
        "enum E{A=F::B as isize}enum F{A=7,B}type A<const N:usize={E::A as usize}>=[u8;N];fn f(){let _:A=[0;8];}",
    ),
    (
        "tuple-alias-field",
        true,
        "type T=usize;struct S(T);type A<const N:usize={S(3).0}>=[u8;N];fn f(){let _:A=[0;3];}",
    ),
    (
        "early-nested-owned-drop",
        false,
        "struct S{n:usize}struct W{e:S}impl Drop for S{fn drop(&mut self){}}type A<const N:usize={W{e:S{n:3}}.e.n}>=[u8;N];fn f(){let _:A=[0;3];}",
    ),
    (
        "generic-cast",
        true,
        "type I<T>=T;type A<const N:usize={3 as I<usize>}>=[u8;N];fn f(){let _:A=[0;3];}",
    ),
    (
        "generic-cast-wrong",
        false,
        "type I<T>=T;type A<const N:usize={259 as I<u8> as usize}>=[u8;N];fn f(){let _:A=[0;259];}",
    ),
    (
        "generic-cast-truncated",
        true,
        "type I<T>=T;type A<const N:usize={259u16 as I<u8> as usize}>=[u8;N];fn f(){let _:A=[0;3];}",
    ),
    (
        "enum-alias-cast",
        true,
        "type I<T>=T;enum E{A=C as I<isize>,B}const C:u8=7;type A<const N:usize={E::B as usize}>=[u8;N];fn f(){let _:A=[0;8];}",
    ),
    (
        "field-value-cycle",
        false,
        "struct S{a:A}type A<const N:usize={S{a:[]}.a[0] as usize}>=[u8;N];",
    ),
    (
        "field-finite-recursion",
        true,
        "struct S{a:A<3>}type A<const N:usize={S{a:[3;3]}.a[0] as usize}>=[u8;N];fn f(){let _:A=[0;3];}",
    ),
    (
        "forward-field-default",
        true,
        "struct S{a:B}type B<const N:usize=3>=[u8;N];type A<const N:usize={S{a:[3;3]}.a[0] as usize}>=[u8;N];fn f(){let _:A=[0;3];}",
    ),
    (
        "recursive-static-read",
        true,
        "#[derive(Copy,Clone)]struct S{r:&'static S,n:usize}static V:S=S{r:&V,n:3};type A<const N:usize={V.r.n}>=[u8;N];fn f(){let _:A=[0;3];}",
    ),
    (
        "later-copy-derive",
        true,
        "#[derive(Copy,Clone)]struct S{n:usize}type A<const N:usize={[S{n:3};3][2].n}>=[u8;N];fn f(){let _:A=[0;3];}",
    ),
    (
        "zero-array-noncopy",
        true,
        "struct S{n:usize}type A<const N:usize={([S{n:3};0],3).1}>=[u8;N];fn f(){let _:A=[0;3];}",
    ),
    (
        "zero-array-drop",
        false,
        "struct S{n:usize}type A<const N:usize={([S{n:3};0],3).1}>=[u8;N];impl Drop for S{fn drop(&mut self){}}",
    ),
];
