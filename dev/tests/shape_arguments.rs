//! Early argument prerequisites and constant capture scopes are independent contracts.
use crate::qualify as rustc;
use crate::semantic;
use rustgm::{
    body, borrow,
    types::{Kind, Primitive, TypeId},
};

type Case = (
    &'static str,
    bool,
    Option<(u64, &'static str)>,
    &'static str,
);

#[test]
fn constant_arguments_preserve_values_and_own_their_lifetime_proofs()
-> Result<(), Box<dyn std::error::Error>> {
    for &(name, accepted, exact, source) in CASES {
        rustc::qualify(name, source, accepted)?;
        if let Some((length, function)) = exact {
            for correct in [true, false] {
                let written = length + u64::from(!correct);
                rustc::qualify(
                    &format!("{name}-exact-{correct}"),
                    &format!("{source}fn exact(){{{function}([0u8;{written}]);}}"),
                    correct,
                )?;
            }
        }
        let result = semantic::analyze(source);
        if !accepted {
            assert!(result.is_err(), "{name}");
            continue;
        }
        let (_, facts) = result.map_err(|e| format!("{name}: {e}"))?;
        let contracts = body::regions::collect(&facts).map_err(|e| format!("{name}: {e}"))?;
        borrow::regions::check(&facts.types, &facts.lifetimes.variances, &contracts)
            .map_err(|e| format!("{name}: {e}"))?;
        if let Some((length, _)) = exact {
            let function = facts.functions.last().ok_or("missing consumer")?;
            let Some(Kind::Function { parameters, .. }) = facts.types.kind(function.signature)
            else {
                return Err("missing consumer signature".into());
            };
            let parameter = facts
                .types
                .list(parameters)
                .and_then(|p| p.first())
                .ok_or("missing consumer parameter")?;
            assert_eq!(
                facts.types.kind(*parameter),
                Some(Kind::Array {
                    element: TypeId::primitive(Primitive::U8),
                    length
                }),
                "{name}"
            );
        }
    }
    Ok(())
}

const CASES: &[Case] = &[
    (
        "late-function",
        true,
        Some((3, "consume")),
        "struct S<T>{v:T,n:usize}type A<const N:usize={S{v:f,n:3}.n}>=[u8;N];fn f(x:u8)->u8{x}fn consume(_:A){}",
    ),
    (
        "late-generic-function",
        true,
        Some((3, "consume")),
        "struct S<T>{v:T,n:usize}type A<const N:usize={S{v:f::<u8>,n:3}.n}>=[u8;N];fn f<T>(x:T)->T{x}fn consume(_:A){}",
    ),
    (
        "early-generic-function",
        true,
        Some((3, "consume")),
        "fn f<T>(x:T)->T{x}struct S<T>{v:T,n:usize}type A<const N:usize={S{v:f::<u8>,n:3}.n}>=[u8;N];fn consume(_:A){}",
    ),
    (
        "inferred-function",
        true,
        Some((3, "consume")),
        "struct S<T>{v:T,n:usize}type A<const N:usize={S::<fn(u8)->u8>{v:f,n:3}.n}>=[u8;N];fn f<T>(x:T)->T{x}fn consume(_:A){}",
    ),
    (
        "function-bound",
        false,
        None,
        "struct X;struct S<T>{v:T,n:usize}type A<const N:usize={S{v:f::<X>,n:3}.n}>=[u8;N];fn f<T:Copy>(x:T)->T{x}fn consume(_:A){}",
    ),
    (
        "type-dependent-default",
        false,
        None,
        "struct W<T>{v:[T;0],n:usize}type A<T,const N:usize={W::<T>{v:[],n:3}.n}>=[T;N];",
    ),
    (
        "const-dependent-default",
        false,
        None,
        "struct W<const N:usize>{n:usize}type A<const M:usize,const N:usize={W::<M>{n:3}.n}>=[u8;N];",
    ),
    (
        "explicit-const-dependency",
        false,
        None,
        "struct W<const N:usize>{n:usize}type A<const N:usize>=[u8;N];fn f<const M:usize>(_:A<{W::<M>{n:3}.n}>){}",
    ),
    (
        "wrong-default-kind",
        false,
        None,
        "struct S<T,const N:usize=true>(T);type A<const N:usize={S(3usize).0}>=[u8;N];",
    ),
    (
        "self-explicit-default",
        true,
        Some((4, "consume")),
        "struct S<T,const N:usize={S::<usize,0>(3).0}>(T);type A<const N:usize={S::<usize>(4).0}>=[u8;N];fn consume(_:A){}",
    ),
    (
        "self-inferred-default",
        true,
        Some((4, "consume")),
        "struct S<T,const N:usize={S::<_,0>(3usize).0}>(T);type A<const N:usize={S::<usize>(4).0}>=[u8;N];fn consume(_:A){}",
    ),
    (
        "self-default-cycle",
        false,
        None,
        "struct S<T,const N:usize={S::<usize>(3).0}>(T);",
    ),
    (
        "nested-default",
        true,
        Some((4, "consume")),
        "struct S<T,const N:usize={R(3).0}>(T);struct R<T>(T);type A<const N:usize={S::<usize>(4).0}>=[u8;N];fn consume(_:A){}",
    ),
    (
        "unused-invalid-default",
        false,
        None,
        "struct S<T,U:Copy=X>(T,U);struct X;type A<const N:usize={S::<usize,u8>(3,0).0}>=[u8;N];",
    ),
    (
        "default-owned-type",
        true,
        Some((3, "f")),
        "struct G{v:u8}struct S<T,U=G>{a:T,v:[U;0]}type A<const N:usize={S::<usize>{a:3,v:[]}.a}>=[u8;N];fn f(_:A){}",
    ),
    (
        "default-derive-type",
        true,
        Some((3, "f")),
        "#[derive(Clone,Copy)]struct G{v:u8}struct S<T,U:Copy=G>{a:T,v:[U;0]}type A<const N:usize={S::<usize>{a:3,v:[]}.a}>=[u8;N];fn f(_:A){}",
    ),
    (
        "alias-default-type",
        true,
        Some((3, "f")),
        "struct G{v:u8}struct S<T,U>{a:T,v:[U;0]}type V<T,U=G>=S<T,U>;type A<const N:usize={V::<usize>{a:3,v:[]}.a}>=[u8;N];fn f(_:A){}",
    ),
    (
        "alias-hole-default",
        true,
        Some((3, "f")),
        "struct G{v:u8}struct S<T,U>{a:T,v:[U;0]}type V<T,U=G>=S<T,U>;type A<const N:usize={V::<_>{a:3usize,v:[]}.a}>=[u8;N];fn f(_:A){}",
    ),
    (
        "parameter-direct",
        true,
        None,
        "struct S<const N:usize>;fn f<const N:usize>(_:S<N>){}",
    ),
    (
        "parameter-block",
        true,
        None,
        "struct S<const N:usize>;fn f<const N:usize>(_:S<{N}>){}",
    ),
    (
        "parameter-parenthesized",
        false,
        None,
        "struct S<const N:usize>;fn f<const N:usize>(_:S<{(N)}>){}",
    ),
    (
        "parameter-nested-block",
        false,
        None,
        "struct S<const N:usize>;fn f<const N:usize>(_:S<{{N}}>){}",
    ),
    (
        "parameter-zero-add",
        false,
        None,
        "struct S<const N:usize>;fn f<const N:usize>(_:S<{N+0}>){}",
    ),
    (
        "lifetime-argument",
        false,
        None,
        "struct W<'a>{v:&'a u8,n:usize}type A<'a,const N:usize={W::<'a>{v:&1,n:3}.n}>=(&'a u8,[u8;N]);",
    ),
    (
        "higher-ranked-function",
        true,
        Some((3, "f")),
        "fn id<'a>(x:&'a u8)->&'a u8{x}struct S<T>{v:T,n:usize}type A<const N:usize={S::<for<'a>fn(&'a u8)->&'a u8>{v:id,n:3}.n}>=[u8;N];fn f(_:A){}",
    ),
    (
        "named-lifetime-type",
        false,
        None,
        "struct S<T>{v:[T;0],n:usize}type A<'a,const N:usize={S::<&'a u8>{v:[],n:3}.n}>=(&'a u8,[u8;N]);",
    ),
    (
        "anonymous-lifetime-type",
        true,
        Some((3, "f")),
        "struct S<T>{v:[T;0],n:usize}type A<const N:usize={S::<&'_ u8>{v:[],n:3}.n}>=[u8;N];fn f(_:A){}",
    ),
    (
        "erased-region",
        true,
        Some((3, "f")),
        "type I<'a>=u8;struct S<T>{v:T}type A<const N:usize={S::<I<'_>>{v:3}.v as usize}>=[u8;N];fn f(_:A){}",
    ),
    (
        "nested-region",
        true,
        Some((4, "f")),
        "struct S<T>{v:[T;0],n:usize}type R<const N:usize>=[u8;N];type A<const M:usize={S::<R<{S::<&'_ u8>{v:[],n:3}.n}>>{v:[],n:4}.n}>=[u8;M];fn f(_:A){}",
    ),
    (
        "outer-binder",
        false,
        None,
        "struct S<T>{v:[T;0],n:usize}struct W<const N:usize>;fn f(_:for<'a>fn(&'a u8,W<{S::<&'a u8>{v:[],n:3}.n}>)){}",
    ),
    (
        "mixed-binders",
        false,
        None,
        "struct S<T>{v:[T;0],n:usize}type A<'a,const N:usize={S::<for<'b>fn(&'b u8,&'a u8)>{v:[],n:3}.n}>=(&'a u8,[u8;N]);",
    ),
    (
        "two-local-binders",
        true,
        Some((3, "f")),
        "struct S<T>{v:[T;0],n:usize}type A<const N:usize={S::<(for<'a>fn(&'a u8),for<'b>fn(&'b u8))>{v:[],n:3}.n}>=[u8;N];fn f(_:A){}",
    ),
    (
        "macro-local-binder",
        true,
        Some((3, "f")),
        "struct S<T>{v:[T;0],n:usize}macro_rules! m{()=>{S::<for<'a>fn(&'a u8)>{v:[],n:3}.n}}type A<const N:usize={m!()}>=[u8;N];fn f(_:A){}",
    ),
    (
        "macro-outer-type",
        false,
        None,
        "struct S<T>{v:[T;0],n:usize}macro_rules! m{($t:ty)=>{S::<$t>{v:[],n:3}.n}}type A<T,const N:usize={m!(T)}>=[T;N];",
    ),
    (
        "late-default-alias",
        true,
        Some((3, "f")),
        "struct G{v:u8}struct S<T,U=G>{a:T,v:[U;0]}type V<T>=S<T>;type A<const N:usize={V::<usize>{a:3,v:[]}.a}>=[u8;N];fn f(_:A){}",
    ),
    (
        "nested-partial-default",
        true,
        Some((3, "f")),
        "struct G{v:u8}struct S<T,U=G>{a:T,v:[U;0]}struct W<T>{v:T}type A<const N:usize={W::<S<_>>{v:S{a:3usize,v:[G{v:1};0]}}.v.a}>=[u8;N];fn f(_:A){}",
    ),
    (
        "erased-outer-lifetime",
        false,
        None,
        "type I<'a>=u8;struct S<T>{v:T}type A<'a,const N:usize={S::<I<'a>>{v:3}.v as usize}>=(&'a u8,[u8;N]);",
    ),
];
