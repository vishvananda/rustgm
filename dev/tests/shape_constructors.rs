//! Early constants reuse ordinary mixed argument inference and declaration facts.
use crate::qualify as rustc;
use crate::semantic;
use rustgm::types::{Kind, Primitive, TypeId};
use std::fmt::Write as _;

use crate::native_references as native;

#[test]
#[ignore = "requires the qualified CPPGM LowIR-to-native backend"]
fn early_constants_reach_native_code_with_their_declared_values()
-> Result<(), Box<dyn std::error::Error>> {
    native::check_cases(
        "early-constant-native",
        &[
            (
                "tuple-default",
                "struct S<T,const N:usize=2>([T;N]);type A<const N:usize={S::<_>([3usize;2]).0[1]}>=[u8;N];fn f(x:A){check(x[2]==7);}fn main(){f([5,6,7]);}",
                false,
            ),
            (
                "alias-variant",
                "enum E<T>{A(T),B}type V<T>=E<T>;struct H<T>{v:T,n:usize}type A<const N:usize={H{v:V::<u8>::A(1),n:3}.n}>=[u8;N];fn f(x:A){check(x[2]==7);}fn main(){f([5,6,7]);}",
                false,
            ),
            (
                "default-derive",
                "#[derive(Clone,Copy)]struct G{v:u8}struct S<T,U:Copy=G>{a:T,v:[U;0]}type A<const N:usize={S::<usize>{a:3,v:[]}.a}>=[u8;N];fn f(x:A){check(x[2]==7);}fn main(){f([5,6,7]);}",
                false,
            ),
            (
                "erased-annotation",
                "type I<'a>=u8;struct S<T>{v:T}type A<const N:usize={S::<I<'_>>{v:3}.v as usize}>=[u8;N];fn f(x:A){check(x[2]==7);}fn main(){f([5,6,7]);}",
                false,
            ),
            (
                "nested-annotation",
                "struct S<T>{v:[T;0],n:usize}type R<const N:usize>=[u8;N];type A<const M:usize={S::<R<{S::<&'_ u8>{v:[],n:3}.n}>>{v:[],n:4}.n}>=[u8;M];fn f(x:A){check(x[3]==7);}fn main(){f([4,5,6,7]);}",
                false,
            ),
            (
                "local-binders",
                "struct S<T>{v:[T;0],n:usize}type A<const N:usize={S::<(for<'a>fn(&'a u8),for<'b>fn(&'b u8))>{v:[],n:3}.n}>=[u8;N];fn f(x:A){check(x[2]==7);}fn main(){f([5,6,7]);}",
                false,
            ),
            (
                "value-negative-control",
                "struct S<T>(T);type A<const N:usize={S(3usize).0}>=[u8;N];fn f(x:A){check(x[2]==8);}fn main(){f([5,6,7]);}",
                true,
            ),
        ],
    )
}

#[test]
fn early_constructor_results_match_independent_exact_lengths()
-> Result<(), Box<dyn std::error::Error>> {
    for &(name, accepted, length, source) in CASES {
        rustc::qualify(name, source, accepted)?;
        if !accepted {
            assert!(semantic::analyze(source).is_err(), "{name}");
            continue;
        }
        for correct in [true, false] {
            let written = length + u64::from(!correct);
            rustc::qualify(
                &format!("{name}-length-{correct}"),
                &format!("{source}fn exact(){{f([0u8;{written}]);}}"),
                correct,
            )?;
        }
        let (_, facts) = semantic::analyze(source).map_err(|e| format!("{name}: {e}"))?;
        let function = facts.functions.last().ok_or("missing array signature")?;
        let Some(Kind::Function { parameters, .. }) = facts.types.kind(function.signature) else {
            return Err("missing function signature".into());
        };
        let parameter = facts
            .types
            .list(parameters)
            .and_then(|p| p.first())
            .ok_or("missing parameter")?;
        assert_eq!(
            facts.types.kind(*parameter),
            Some(Kind::Array {
                element: TypeId::primitive(Primitive::U8),
                length
            }),
            "{name}",
        );
    }
    Ok(())
}

#[test]
fn repeated_shape_defaults_reuse_completed_nominal_instances()
-> Result<(), Box<dyn std::error::Error>> {
    let mut source = String::from("struct S<T,const N:usize=2>([T;N]);fn expected(_: [u8;3]){}");
    for index in 0..512 {
        write!(
            source,
            "type A{index}<const N:usize={{S::<_>([3usize;2]).0[1]}}>=[u8;N];fn f{index}(_:A{index}){{}}"
        )?;
    }
    let (_, facts) = semantic::analyze(&source)?;
    let expected = facts.functions.first().ok_or("missing control")?.signature;
    assert_eq!(facts.functions.len(), 513);
    assert!(facts.functions.iter().all(|f| f.signature == expected));
    assert!(facts.aggregates.instantiation_steps() < 64);
    Ok(())
}

#[test]
fn early_type_arguments_share_declaration_dependencies() -> Result<(), Box<dyn std::error::Error>> {
    let count = 512;
    let mut source = String::from("#![recursion_limit=\"4096\"]");
    for index in 0..count {
        let field = if index + 1 == count {
            "T".to_owned()
        } else {
            format!("S{}<T>", index + 1)
        };
        write!(source, "struct S{index}<T>{{v:{field}}}")?;
    }
    source.push_str("struct W<T>{v:[T;0],n:usize}");
    for index in 0..count {
        write!(
            source,
            "type A{index}<const N:usize={{W::<S{index}<u8>>{{v:[],n:3}}.n}}>=[u8;N];fn f{index}(_:A{index}){{}}"
        )?;
    }
    rustc::qualify("early-shared-declarations", &source, true)?;
    let (_, facts) = semantic::analyze(&source)?;
    assert!(
        facts.aggregates.instantiation_steps() < count * 64,
        "{} field instantiation steps",
        facts.aggregates.instantiation_steps()
    );
    assert!(
        facts.types.len() < count * 32,
        "{} types",
        facts.types.len()
    );
    Ok(())
}

const CASES: &[(&str, bool, u64, &str)] = &[
    (
        "early-nominals-v1-record-infer",
        true,
        3,
        "struct S<T>{v:T}type A<const N:usize={S{v:3usize}.v}>=[u8;N];fn f(_:A){}",
    ),
    (
        "early-nominals-v1-record-explicit",
        true,
        3,
        "struct S<T>{v:T}type A<const N:usize={S::<usize>{v:3}.v}>=[u8;N];fn f(_:A){}",
    ),
    (
        "early-nominals-v1-tuple-infer",
        true,
        3,
        "struct S<T>(T);type A<const N:usize={S(3usize).0}>=[u8;N];fn f(_:A){}",
    ),
    (
        "early-nominals-v1-tuple-explicit",
        true,
        3,
        "struct S<T>(T);type A<const N:usize={S::<usize>(3).0}>=[u8;N];fn f(_:A){}",
    ),
    (
        "early-nominals-v1-tuple-hole",
        true,
        3,
        "struct S<T>(T);type A<const N:usize={S::<_>(3usize).0}>=[u8;N];fn f(_:A){}",
    ),
    (
        "early-nominals-v1-mixed-array",
        true,
        3,
        "struct S<T,const N:usize>([T;N]);type A<const N:usize={S([3u8;2]).0[0] as usize}>=[u8;N];fn f(_:A){}",
    ),
    (
        "early-nominals-v1-partial-default",
        true,
        4,
        "struct S<T,U=T>(T,U);type A<const N:usize={S::<usize>(3,4).1}>=[u8;N];fn f(_:A){}",
    ),
    (
        "early-nominals-v1-partial-hole",
        true,
        4,
        "struct S<T,U=T>(T,U);type A<const N:usize={S::<_>(3usize,4).1}>=[u8;N];fn f(_:A){}",
    ),
    (
        "early-nominals-v1-alias-record",
        true,
        3,
        "struct S<T>{v:T}type R<T>=S<T>;type A<const N:usize={R::<usize>{v:3}.v}>=[u8;N];fn f(_:A){}",
    ),
    (
        "early-nominals-v1-cast-field",
        true,
        3,
        "struct S<T>{v:T}type A<const N:usize={(S{v:3u8}.v as usize)}>=[u8;N];fn f(_:A){}",
    ),
    (
        "early-nominals-v2-enum-unit",
        true,
        3,
        "struct H<T>{v:T,n:usize}enum E<T>{A(T),B}type R<const N:usize={H{v:E::<u8>::B,n:3}.n}>=[u8;N];fn f(_:R){}",
    ),
    (
        "early-nominals-v2-enum-tuple",
        true,
        3,
        "struct H<T>{v:T,n:usize}enum E<T>{A(T),B}type R<const N:usize={H{v:E::A(1u8),n:3}.n}>=[u8;N];fn f(_:R){}",
    ),
    (
        "early-nominals-v2-alias-unit",
        true,
        3,
        "struct H<T>{v:T,n:usize}enum E<T>{A(T),B}type V<T>=E<T>;type R<const N:usize={H{v:V::<u8>::B,n:3}.n}>=[u8;N];fn f(_:R){}",
    ),
    (
        "early-nominals-v2-alias-tuple",
        true,
        3,
        "struct H<T>{v:T,n:usize}enum E<T>{A(T),B}type V<T>=E<T>;type R<const N:usize={H{v:V::<u8>::A(1),n:3}.n}>=[u8;N];fn f(_:R){}",
    ),
    (
        "early-nominals-v2-alias-record",
        true,
        3,
        "struct H<T>{v:T,n:usize}enum E<T>{A{v:T},B}type V<T>=E<T>;type R<const N:usize={H{v:V::<u8>::A{v:1},n:3}.n}>=[u8;N];fn f(_:R){}",
    ),
    (
        "early-nominals-v2-alias-variant-arguments",
        false,
        3,
        "struct H<T>{v:T,n:usize}enum E<T>{A(T),B}type V<T>=E<T>;type R<const N:usize={H{v:V::<u8>::A::<u8>(1),n:3}.n}>=[u8;N];fn f(_:R){}",
    ),
    (
        "early-nominals-v2-forward-default",
        true,
        4,
        "type R<const N:usize={S::<usize>(3,4).1}>=[u8;N];struct S<T,U=T>(T,U);fn f(_:R){}",
    ),
    (
        "early-nominals-v2-default-infers",
        true,
        3,
        "struct S<T=u8>(T);type R<const N:usize={S(3usize).0}>=[u8;N];fn f(_:R){}",
    ),
    (
        "early-nominals-v2-default-empty-infers",
        true,
        3,
        "struct S<T=u8>(T);type R<const N:usize={S::<>(3usize).0}>=[u8;N];fn f(_:R){}",
    ),
    (
        "early-nominals-v2-partial-default-mismatch",
        false,
        3,
        "struct S<T,U=u8>(T,U);type R<const N:usize={S::<usize>(3,4usize).1}>=[u8;N];fn f(_:R){}",
    ),
    (
        "early-nominals-v2-partial-explicit-hole",
        true,
        4,
        "struct S<T,U=u8>(T,U);type R<const N:usize={S::<usize,_>(3,4usize).1}>=[u8;N];fn f(_:R){}",
    ),
    (
        "early-nominals-v2-partial-length",
        true,
        4,
        "struct S<T,const N:usize=3>([T;N]);type R<const N:usize={S::<usize>([4;3]).0[1]}>=[u8;N];fn f(_:R){}",
    ),
    (
        "early-nominals-v2-partial-length-mismatch",
        false,
        3,
        "struct S<T,const N:usize=3>([T;N]);type R<const N:usize={S::<usize>([4;2]).0[1]}>=[u8;N];fn f(_:R){}",
    ),
    (
        "early-nominals-v2-inferred-lifetime",
        true,
        3,
        "struct S<'a,T>(&'a T);type R<const N:usize={*S(&3usize).0}>=[u8;N];fn f(_:R){}",
    ),
    (
        "early-nominals-v2-explicit-lifetime",
        true,
        3,
        "struct S<'a,T>(&'a T);type R<const N:usize={*S::<'static,usize>(&3).0}>=[u8;N];fn f(_:R){}",
    ),
    (
        "early-nominals-v2-alias-default",
        true,
        3,
        "struct S<T>{v:T}type V<T=usize>=S<T>;type R<const N:usize={V{v:3}.v}>=[u8;N];fn f(_:R){}",
    ),
    (
        "early-nominals-v2-alias-partial",
        true,
        3,
        "struct S<T,U>{v:T,n:U}type V<T,U=T>=S<T,U>;type R<const N:usize={V::<_>{v:1usize,n:3}.n}>=[u8;N];fn f(_:R){}",
    ),
    (
        "early-nominals-v2-named-constant",
        true,
        3,
        "struct S<T>(T);const V:S<usize>=S(3);type R<const N:usize={V.0}>=[u8;N];fn f(_:R){}",
    ),
    (
        "early-nominals-v2-named-static",
        true,
        3,
        "struct S<T>(T);static V:S<usize>=S(3);type R<const N:usize={V.0}>=[u8;N];fn f(_:R){}",
    ),
    (
        "early-nominals-v2-derive-copy",
        true,
        3,
        "#[derive(Clone,Copy)]struct S<T>{v:T}struct W<T:Copy>{v:T}type R<const N:usize={W{v:S{v:3usize}}.v.v}>=[u8;N];fn f(_:R){}",
    ),
    (
        "early-nominals-v2-packed-reference",
        true,
        3,
        "#[repr(align(8))]struct Al(u8);#[repr(packed)]struct P<T>{a:T,b:u8}type R<const N:usize={*(&P{a:Al(2),b:3}.b) as usize}>=[u8;N];fn f(_:R){}",
    ),
    (
        "early-nominals-v2-false-branch-bound",
        false,
        3,
        "struct X;struct S<T:Copy>(T);struct W<T>{v:T,n:usize}type R<const N:usize={if true{3}else{W{v:S(X),n:3}.n}}>=[u8;N];fn f(_:R){}",
    ),
    (
        "early-packing-v1-array",
        true,
        3,
        "#[repr(align(8))]struct Al(u8);#[repr(packed)]struct P{a:[Al;1],b:u8}type A<const N:usize={*(&P{a:[Al(2)],b:3}.b) as usize}>=[u8;N];fn f(_:A){}",
    ),
    (
        "early-packing-v1-argument",
        true,
        3,
        "#[repr(align(8))]struct Al(u8);#[repr(packed)]struct P<T>{a:T,b:u8}type A<const N:usize={*(&P{a:Al(2),b:3}.b) as usize}>=[u8;N];fn f(_:A){}",
    ),
    (
        "early-packing-v1-direct",
        false,
        3,
        "#[repr(align(8))]struct Al(u8);#[repr(packed)]struct P{a:Al,b:u8}type A<const N:usize={*(&P{a:Al(2),b:3}.b) as usize}>=[u8;N];fn f(_:A){}",
    ),
];
