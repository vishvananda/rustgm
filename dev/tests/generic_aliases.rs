use crate::qualify as rustc;
use crate::semantic;
use rustgm::types::{Kind, Primitive, TypeId};
use std::fmt::Write as _;

const ACCEPTED: &[(&str, &str)] = &[
    (
        "private-field-in-caller",
        "type A<const N:usize>=[u8;N];mod m{struct S{n:usize}fn f(){let _:super::A<{S{n:3}.n}>;}}",
    ),
    (
        "const-query-storage",
        "type A<const N:usize>=[u8;N];fn f(_:A<{1+2}>,_:A<{3+2}>,_:A<{C}>,_:A<{D}>){}const C:usize=7;const D:usize=9;",
    ),
    (
        "const-query-modules",
        "type A<const N:usize>=[u8;N];mod m{pub type B=super::A<{1+2}>;}fn f(_:m::B,_:A<{5+1}>){}",
    ),
    (
        "bare-const-name",
        "const C:usize=3;type A<const N:usize>=[u8;N];fn f(x:A<C>)->[u8;3]{x}",
    ),
    (
        "const-type-ambiguity-block",
        "const N:usize=3;type N=u8;type A<const C:usize>=[u8;C];fn f(x:A<{N}>)->[u8;3]{x}",
    ),
    ("unused-array-shape", "type A<T>=([str;1],T);"),
    ("unused-default-shape", "type A<T=[str;1]>=T;"),
    (
        "unused-nested-shape",
        "type A<T>=[T;1];type B<U>=(A<str>,U);",
    ),
    ("unsized-default", "type A<T=str>=T;fn f(x:&A)->&str{x}"),
    (
        "lifetime-default",
        "type A<'a,T=&'a u8>=T;fn f<'b>(x:A<'b>)->&'b u8{x}",
    ),
    (
        "const-name",
        "const C:usize=3;type A<const N:usize>=[u8;N];fn f(x:A<{C}>)->[u8;3]{x}",
    ),
    (
        "forward-const-name",
        "type A<const N:usize>=[u8;N];fn f(x:A<{C}>)->[u8;3]{x}const C:usize=D;const D:usize=3;",
    ),
    ("identity", "type A<T>=T;fn f(x:A<u16>)->u16{x}"),
    (
        "nested",
        "type A<T>=(T,T);type B<U>=A<U>;fn f(x:B<u8>)->(u8,u8){x}",
    ),
    ("type-default", "type A<T=u8>=T;fn f(x:A)->u8{x}"),
    (
        "dependent-default",
        "type A<T,U=T>=(T,U);fn f(x:A<u16>)->(u16,u16){x}",
    ),
    (
        "array",
        "type A<T,const N:usize>=[T;N];fn f(x:A<u8,4>)->[u8;4]{x}",
    ),
    (
        "const-forward",
        "type A<T,const N:usize>=[T;N];type B<U,const M:usize>=A<U,M>;fn f(x:B<u8,3>)->[u8;3]{x}",
    ),
    (
        "const-block",
        "type A<T,const N:usize>=[T;{N}];type B<U,const M:usize>=A<U,{M}>;fn f(x:B<u8,3>)->[u8;3]{x}",
    ),
    (
        "const-default",
        "type A<T=u8,const N:usize=4>=[T;N];fn f(x:A)->[u8;4]{x}",
    ),
    (
        "const-dependent-default",
        "type A<const N:usize,const M:usize=N>=([u8;N],[u8;M]);fn f(x:A<3>)->([u8;3],[u8;3]){x}",
    ),
    (
        "const-evaluation",
        "type A<const N:usize>=[u8;N];fn f(x:A<{1+2}>)->[u8;3]{x}",
    ),
    (
        "scalar-unused-const",
        "type A<T,const B:bool,const C:char,const N:i8>=T;fn f(x:A<u8,true,'x',-128>)->u8{x}",
    ),
    (
        "lifetime",
        "type A<'a,T>=&'a T;fn f<'b>(x:A<'b,u8>)->&'b u8{x}",
    ),
    (
        "lifetime-elision",
        "type A<'a,T>=&'a T;fn f(x:A<u8>)->&u8{x}",
    ),
    (
        "callable",
        "type A<T>=fn(T)->T;fn f(x:A<u8>)->fn(u8)->u8{x}",
    ),
    (
        "nested-binder",
        "type A<T>=for<'a> fn(&'a u8,T)->T;fn f<'b>(x:A<&'b u8>)->for<'a> fn(&'a u8,&'b u8)->&'b u8{x}",
    ),
    ("unsized-alias", "type A<T>=T;fn f(x:&A<str>)->&str{x}"),
    (
        "unused-lifetime",
        "type A<'a,T>=T;fn f(x:A<'static,u8>)->u8{x}",
    ),
    (
        "configured",
        "type A<#[cfg(any())] Gone,T>=T;fn f(x:A<u8>)->u8{x}",
    ),
    ("module", "mod m{pub type A<T>=T;}fn f(x:m::A<u8>)->u8{x}"),
];
const REJECTED: &[(&str, &str)] = &[
    (
        "alias-cannot-grant-field-access",
        "mod m{pub struct S{n:usize}pub type A<const N:usize>=[u8;N];}fn f(){let _:m::A<{m::S{n:3}.n}>;}",
    ),
    (
        "alias-cannot-grant-const-field-access",
        "mod m{pub struct S{n:usize}pub const C:S=S{n:3};pub type A<const N:usize>=[u8;N];}fn f(){let _:m::A<{m::C.n}>;}",
    ),
    (
        "const-type-ambiguity",
        "const N:usize=3;type N=u8;type A<const C:usize>=[u8;C];fn f(_:A<N>){}",
    ),
    ("nested-const-block", "type A<const N:usize>=[u8;{{N}}];"),
    ("grouped-const-parameter", "type A<const N:usize>=[u8;(N)];"),
    ("used-array-shape", "type A<T>=([str;1],T);fn f(_:&A<u8>){}"),
    ("used-default-shape", "type A<T=[str;1]>=T;fn f(_:&A){}"),
    ("missing", "type A<T>=T;fn f(_:A){}"),
    ("extra", "type A<T>=T;fn f(_:A<u8,u16>){}"),
    ("wrong-kind", "type A<T>=T;fn f(_:A<3>){}"),
    (
        "wrong-const-kind",
        "type A<const N:usize>=[u8;N];fn f(_:A<u8>){}",
    ),
    (
        "wrong-const-type",
        "type A<const N:usize>=[u8;N];fn f(_:A<3u8>){}",
    ),
    ("unused-type", "type A<T>=u8;"),
    (
        "unused-through-alias",
        "type A<T,const N:usize>=T;type B<T>=A<u8,3>;",
    ),
    ("invalid-const-type", "type A<const N:f32>=u8;"),
    ("generic-const-type", "type A<T,const N:T>=T;"),
    ("default-order", "type A<T=u8,U>=(T,U);"),
    ("default-forward", "type A<T=U,U=u8>=(T,U);"),
    ("default-self", "type A<T=T>=T;"),
    ("cycle", "type A<T>=B<T>;type B<U>=A<U>;"),
    ("duplicate", "type A<T,const T:usize>=(T,);"),
    ("lifetime-order", "type A<T,'a>=&'a T;"),
    ("wrong-array-const", "type A<const N:u8>=[u8;N];"),
    ("const-as-type", "type A<const N:usize>=N;"),
    ("const-overflow", "type A<const N:u8>=u8;fn f(_:A<256>){}"),
    (
        "unsigned-negative",
        "type A<const N:usize>=[u8;N];fn f(_:A<-1>){}",
    ),
    ("ill-formed-expansion", "type A<T>=[T;1];fn f(_:&A<str>){}"),
    ("invalid-unused-default", "type A<const N:u8=256>=u8;"),
    (
        "const-arithmetic-parameter",
        "type A<const N:usize>=[u8;{N+1}];",
    ),
];
#[test]
fn source_aliases_substitute_types_lifetimes_consts_and_defaults() -> Result<(), String> {
    for &(name, source) in ACCEPTED {
        semantic::analyze(source).map_err(|e| format!("{name}: {e}"))?;
    }
    Ok(())
}
#[test]
fn invalid_alias_declarations_and_arguments_are_rejected_even_when_unused() {
    for &(name, source) in REJECTED {
        assert!(semantic::analyze(source).is_err(), "{name}: {source}");
    }
}
#[test]
fn alias_outcomes_are_independently_qualified() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, source) in ACCEPTED {
        rustc::qualify(&format!("generic-alias-accept-{name}"), source, true)?;
    }
    for &(name, source) in REJECTED {
        rustc::qualify(&format!("generic-alias-reject-{name}"), source, false)?;
    }
    Ok(())
}
#[test]
fn concrete_aliases_share_existing_canonical_array_identity() -> Result<(), String> {
    let (_, facts) =
        semantic::analyze("type A<T,const N:usize>=[T;N];fn a(_:A<u8,3>){}fn b(_:[u8;3]){}")?;
    assert_eq!(facts.functions[0].signature, facts.functions[1].signature);
    let Some(Kind::Function { parameters, .. }) = facts.types.kind(facts.functions[0].signature)
    else {
        return Err("missing signature".into());
    };
    let ty = facts
        .types
        .list(parameters)
        .and_then(|p| p.first())
        .ok_or("missing argument")?;
    assert_eq!(
        facts.types.kind(*ty),
        Some(Kind::Array {
            element: TypeId::primitive(Primitive::U8),
            length: 3
        })
    );
    assert_eq!(facts.types.contains_parameters(*ty), Some(false));
    Ok(())
}
#[test]
fn forward_generic_alias_chains_use_the_iterative_declaration_graph() -> Result<(), String> {
    let mut source = String::new();
    for index in 0..2048 {
        write!(source, "type A{index}<T>=A{}<T>;", index + 1).map_err(|e| e.to_string())?;
    }
    source.push_str("type A2048<T>=T;fn f(x:A0<u8>)->u8{x}");
    let (_, facts) = semantic::analyze(&source)?;
    let Some(Kind::Function { result, .. }) = facts.types.kind(facts.functions[0].signature) else {
        return Err("missing signature".into());
    };
    assert_eq!(result, TypeId::primitive(Primitive::U8));
    Ok(())
}
