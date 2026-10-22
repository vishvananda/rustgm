//! Declared outcomes are independently qualified before the frontend is compared.
use crate::qualify as rustc;
use crate::semantic;
use std::fmt::Write as _;
const CASES: &[(&str, bool, &str)] = &[
    (
        "default-static-reference-bound",
        true,
        "struct S<T:'static=u8>where &'static mut T:Copy{v:T}fn main(){}",
    ),
    (
        "default-static-reference-bound-used",
        false,
        "struct S<T:'static=u8>where &'static mut T:Copy{v:T}fn force(_:S){}fn main(){}",
    ),
    (
        "default-static-reference-value",
        false,
        "struct S<T:Copy=&'static mut u8>{v:T}fn main(){}",
    ),
    ("static-type-missing", false, "struct S<T>{v:&'static T}"),
    (
        "static-type-explicit",
        true,
        "struct S<T:'static>{v:&'static T}",
    ),
    (
        "static-region-missing",
        false,
        "struct S<'a>{v:&'static &'a u8}",
    ),
    (
        "static-region-explicit",
        true,
        "struct S<'a:'static>{v:&'static &'a u8}",
    ),
    ("named-type-implied", true, "struct S<'a,T>{v:&'a T}"),
    (
        "nested-static-missing",
        false,
        "struct A<T:'static>{v:T}struct B<T>{v:A<T>}",
    ),
    (
        "nested-static-explicit",
        true,
        "struct A<T:'static>{v:T}struct B<T:'static>{v:A<T>}",
    ),
    (
        "static-transitive",
        true,
        "struct S<'a:'static,T>{v:&'a T,w:&'static T}",
    ),
    (
        "static-transitive-wrong",
        false,
        "struct S<'a,T:'a>{v:&'a T,w:&'static T}",
    ),
    (
        "named-nested-implied",
        true,
        "struct A<'a,T>{v:&'a T}struct B<'a,T>{v:A<'a,T>}",
    ),
    (
        "nested-argument-wf",
        true,
        "struct A<T>{v:*const T}struct B<'a,'b>{v:A<&'a &'b u8>}",
    ),
    (
        "nested-argument-static",
        false,
        "struct A<T>{v:*const T}struct B<'a>{v:A<&'static &'a u8>}",
    ),
    (
        "field-binder-local",
        true,
        "struct S<T>{v:for<'a> fn(&'a T)}",
    ),
    (
        "field-binder-static",
        false,
        "struct S<T>{v:fn(&'static T)}",
    ),
    (
        "parameter-substitution-static",
        false,
        "struct A<T:'static>{v:T}struct B<'a>{v:A<&'a u8>}",
    ),
    (
        "parameter-substitution-static-bound",
        true,
        "struct A<T:'static>{v:T}struct B<'a:'static>{v:A<&'a u8>}",
    ),
    (
        "type-bound-pointer-implied",
        true,
        "struct A<'a,T:'a>{v:*const T,r:&'a u8}struct B<'a,T>{v:A<'a,T>}",
    ),
    (
        "recursive-remap",
        true,
        "struct A<'a,'b,T,U>{p:*const B<'b,'a,U,T>,v:&'a T}struct B<'a,'b,T,U>{p:*const A<'b,'a,U,T>,v:&'a T}",
    ),
    ("plain-both", true, "#[derive(Clone,Copy)]struct S<T>{v:T}"),
    (
        "pointer-both",
        true,
        "#[derive(Clone,Copy)]struct S<T>{v:*const T}",
    ),
    (
        "clone-mut-reference",
        false,
        "#[derive(Clone)]struct S<'a,T>{v:&'a mut T}",
    ),
    (
        "copy-mut-reference",
        false,
        "#[derive(Clone,Copy)]struct S<'a,T>{v:&'a mut T}",
    ),
    (
        "packed-clone",
        true,
        "#[repr(packed)]#[derive(Clone)]struct S<T>{v:T}",
    ),
    (
        "nested-packed-both",
        false,
        "#[repr(packed)]#[derive(Clone,Copy)]struct A<T>{v:T}#[derive(Clone,Copy)]struct S<T>{v:A<T>}",
    ),
    (
        "nested-packed-clone",
        false,
        "#[repr(packed)]#[derive(Clone,Copy)]struct A<T>{v:T}#[derive(Clone)]struct S<T>{v:A<T>}",
    ),
    (
        "nested-packed-declared",
        true,
        "#[repr(packed)]#[derive(Clone,Copy)]struct A<T>{v:T}#[derive(Clone,Copy)]struct S<T:Copy>{v:A<T>}",
    ),
    (
        "nested-packed-equality",
        false,
        "#[repr(packed)]#[derive(Clone,Copy,PartialEq)]struct A<T>{v:T}#[derive(PartialEq)]struct S<T>{v:A<T>}",
    ),
    (
        "nested-packed-equality-bound",
        true,
        "#[repr(packed)]#[derive(Clone,Copy,PartialEq)]struct A<T>{v:T}#[derive(PartialEq)]struct S<T:Copy>{v:A<T>}",
    ),
    (
        "packed-equality",
        true,
        "#[repr(packed)]#[derive(PartialEq)]struct S<T>{v:T}",
    ),
    (
        "float-eq",
        false,
        "#[derive(PartialEq,Eq)]struct S<T>{v:T,f:f32}",
    ),
    (
        "float-partial",
        true,
        "#[derive(PartialEq)]struct S<T>{v:T,f:f32}",
    ),
    (
        "reference-equality-recursive",
        true,
        "#[derive(PartialEq,Eq)]struct S<'a,T>{p:&'a S<'a,[T;1]>,v:T}",
    ),
    (
        "transparent-one",
        true,
        "#[repr(transparent)]struct S<T>(T);",
    ),
    (
        "transparent-two",
        false,
        "#[repr(transparent)]struct S<T,U>(T,U);",
    ),
    (
        "transparent-unit",
        true,
        "#[repr(transparent)]struct S<T>(T,());",
    ),
    (
        "transparent-zero-own",
        false,
        "#[repr(transparent)]struct S<T>(T,[T;0]);",
    ),
    (
        "transparent-zero-u8",
        true,
        "#[repr(transparent)]struct S<T>(T,[u8;0]);",
    ),
    (
        "transparent-zero-u64",
        false,
        "#[repr(transparent)]struct S<T>(T,[u64;0]);",
    ),
    (
        "transparent-zero-param-alone",
        true,
        "#[repr(transparent)]struct S<T>([T;0]);",
    ),
    (
        "transparent-symbolic-array",
        true,
        "#[repr(transparent)]struct S<T,const N:usize>([T;N]);",
    ),
    (
        "transparent-symbolic-array-second",
        false,
        "#[repr(transparent)]struct S<T,const N:usize>(T,[u8;N]);",
    ),
    (
        "transparent-wrapper-unit",
        true,
        "struct Z<T>{v:[*const T;0]}#[repr(transparent)]struct S<T>(Z<T>);",
    ),
    (
        "packed-known-aligned",
        false,
        "#[repr(align(16))]struct A(u8);#[repr(packed)]struct S<T>(T,A);",
    ),
    (
        "packed-known-wrapper-aligned",
        true,
        "struct W<T>(T);#[repr(align(16))]struct A(u8);#[repr(packed)]struct S<T>(T,W<A>);",
    ),
    (
        "packed-zero-aligned",
        true,
        "#[repr(align(16))]struct A;#[repr(packed)]struct S<T>(T,[A;0]);",
    ),
    (
        "packed-pointer-aligned",
        true,
        "#[repr(align(16))]struct A;#[repr(packed)]struct S<T>(T,*const A);",
    ),
    (
        "transparent-dependent-zero",
        true,
        "struct W<T>(T);#[repr(transparent)]struct S<T>(T,W<()>);",
    ),
    (
        "transparent-param-wrapper-zero",
        false,
        "struct W<T>([T;0]);#[repr(transparent)]struct S<T>(T,W<T>);",
    ),
    (
        "transparent-packed-zero-param",
        false,
        "#[repr(packed)]struct Z<T>([T;0]);#[repr(transparent)]struct S<T>(T,Z<T>);",
    ),
    (
        "transparent-const-unit",
        false,
        "struct Z<const N:usize>([();N]);#[repr(transparent)]struct S<T,const N:usize>(T,Z<N>);",
    ),
    (
        "transparent-const-u8",
        false,
        "struct Z<const N:usize>([u8;N]);#[repr(transparent)]struct S<T,const N:usize>(T,Z<N>);",
    ),
    (
        "transparent-zero-generic-pointer",
        false,
        "struct Z<T>([*const T;0]);#[repr(transparent)]struct S<T>(T,Z<T>);",
    ),
    (
        "transparent-zero-c-unit",
        false,
        "#[repr(C)]struct Z;#[repr(transparent)]struct S<T>(T,Z);",
    ),
    (
        "transparent-zero-c-array",
        false,
        "#[repr(C)]struct Z;#[repr(transparent)]struct S<T>(T,[Z;0]);",
    ),
    (
        "packed-array-one",
        true,
        "#[repr(align(16))]struct A;#[repr(packed)]struct S<T>(T,[A;1]);",
    ),
    (
        "packed-tuple",
        true,
        "#[repr(align(16))]struct A;#[repr(packed)]struct S<T>(T,(A,));",
    ),
    (
        "packed-enum-aligned",
        true,
        "#[repr(align(16))]enum A{X}#[repr(packed)]struct S<T>(T,A);",
    ),
    (
        "packed-struct-enum-aligned",
        true,
        "#[repr(align(16))]struct A;enum E{X(A)}#[repr(packed)]struct S<T>(T,E);",
    ),
    (
        "default-self-bound",
        false,
        "struct N;struct S<T:Copy=N>(T);",
    ),
    (
        "default-nested-bound",
        false,
        "struct N;struct A<T:Copy>(T);struct S<T=A<N>>(T);",
    ),
    (
        "alias-default-nested-bound",
        true,
        "struct N;struct A<T:Copy>(T);type S<T=A<N>>=T;",
    ),
    ("default-relaxed", false, "struct S<T=[u8]>(T);"),
    ("default-static", true, "struct S<T,U=&'static T>(T,U);"),
    (
        "default-static-bound",
        true,
        "struct S<T:'static,U=&'static T>(T,U);",
    ),
    ("default-named", true, "struct S<'a,T,U=&'a T>(&'a T,U);"),
    (
        "default-bound-parameter",
        true,
        "struct S<T,U:Copy=T>(T,U);",
    ),
    (
        "default-bound-parameter-valid",
        true,
        "struct S<T:Copy,U:Copy=T>(T,U);",
    ),
    (
        "generic-nested-default",
        true,
        "struct A<T:Copy>(T);struct S<T,U=A<T>>(T,U);",
    ),
    (
        "generic-sized-default",
        true,
        "struct S<T:?Sized,U=T>{v:*const T,w:U}",
    ),
    (
        "default-closed-false-where",
        false,
        "struct N;struct S<T=N>(T)where T:Copy;",
    ),
    (
        "default-closed-compound",
        false,
        "struct N;struct S<T=N>(T)where [T;1]:Copy;",
    ),
    (
        "default-partial-closed",
        false,
        "struct N;struct S<T,U:Copy=N>(T,U);",
    ),
    (
        "default-dependent-concrete",
        true,
        "struct N;struct S<T=N,U:Copy=T>(T,U);",
    ),
    (
        "default-dependent-valid",
        true,
        "struct S<T=u8,U:Copy=T>(T,U);",
    ),
    (
        "default-nested-slice",
        false,
        "struct A<T>(T);struct S<T=A<[u8]>>(T);",
    ),
    (
        "default-direct-slice",
        false,
        "struct S<T=[u8]>{v:*const T}",
    ),
    (
        "default-relaxed-slice",
        true,
        "struct S<T:?Sized=[u8]>{v:*const T}",
    ),
    (
        "lifetime-mut-default",
        true,
        "struct S<'a,T:Copy=&'a mut u8>(&'a u8,T);",
    ),
    (
        "generic-mut-default",
        true,
        "struct S<'a,T,U:Copy=&'a mut T>(&'a T,U);",
    ),
    (
        "lifetime-shared-default",
        true,
        "struct S<'a,T:Copy=&'a u8>(&'a u8,T);",
    ),
    (
        "default-nested-partial",
        true,
        "struct N;struct A<T:Copy>(T);struct S<T,U=(T,A<N>)>(T,U);",
    ),
    (
        "default-array-relaxed-generic",
        true,
        "struct S<T:?Sized,U=[T;1]>{v:*const T,w:U}",
    ),
    (
        "bound-substitution-defaults",
        true,
        "struct N;struct S<T=N,U=N>(T,U)where(T,U):Copy;",
    ),
    (
        "bound-composite-partial-defaults",
        true,
        "struct N;struct S<T,U=N>(T,U)where(T,U):Copy;",
    ),
];
#[test]
fn scoped_nominal_declarations_match_independently_qualified_source()
-> Result<(), Box<dyn std::error::Error>> {
    for &(name, accepted, source) in CASES {
        rustc::qualify(name, source, accepted)?;
        let result = semantic::analyze(source);
        assert_eq!(
            result.is_ok(),
            accepted,
            "{name}: {}",
            result.err().unwrap_or_default()
        );
    }
    Ok(())
}

#[test]
fn forward_symbolic_fields_do_not_expand_every_template_instance() -> Result<(), String> {
    let mut source = String::new();
    for index in 0..4096 {
        let next = if index == 4095 {
            "T".to_owned()
        } else {
            format!("S{}<T>", index + 1)
        };
        write!(source, "struct S{index}<T>{{value:{next}}}").map_err(|e| e.to_string())?;
    }
    let (_, facts) = semantic::analyze(&source)?;
    assert!(facts.types.len() < 30_000, "{}", facts.types.len());
    assert_eq!(facts.aggregates.instantiation_steps(), 0);
    assert!(facts.layouts.is_empty());
    Ok(())
}
