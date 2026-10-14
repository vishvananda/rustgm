//! Constructor expectations are qualified independently before candidate checking.
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;
const CASES: &[(&str, bool, &str)] = &[
    (
        "tuple-infer",
        true,
        "struct S<T>(T);fn main(){let s=S(1u8);let _:u8=s.0;}",
    ),
    (
        "record-infer",
        true,
        "struct S<T>{v:T}fn main(){let s=S{v:1u8};let _:u8=s.v;}",
    ),
    (
        "enum-infer",
        true,
        "enum E<T>{X(T),Y}fn main(){let s=E::X(1u8);let _:E<u8>=s;}",
    ),
    (
        "enum-unit-infer",
        true,
        "enum E<T>{X(T),Y}fn main(){let s=E::Y;let _:E<u8>=s;}",
    ),
    (
        "defaults-omitted",
        true,
        "struct S<T=u8>(T);fn main(){let s=S(1u16);let _:S<u16>=s;}",
    ),
    (
        "defaults-empty",
        true,
        "struct S<T=u8>(T);fn main(){let s=S::<>(1u16);let _:S<u16>=s;}",
    ),
    (
        "defaults-partial",
        false,
        "struct S<T,U=u8>(T,U);fn main(){let _s=S::<u16>(1u16,2u32);}",
    ),
    (
        "defaults-partial-valid",
        true,
        "struct S<T,U=u8>(T,U);fn main(){let _s=S::<u16>(1u16,2u8);}",
    ),
    (
        "defaults-prefix-dependent",
        true,
        "struct S<T,U=T>(T,U);fn main(){let _s=S::<u16>(1u16,2u16);}",
    ),
    (
        "defaults-prefix-hole",
        true,
        "struct S<T,U=T>(T,U);fn main(){let _s=S::<_>(1u16,2u16);}",
    ),
    (
        "defaults-explicit-hole",
        true,
        "struct S<T,U=u8>(T,U);fn main(){let _s=S::<u16,_>(1u16,2u32);}",
    ),
    (
        "defaults-lifetime-only",
        true,
        "struct S<'a,T=u8>(&'a T);fn main(){let n=1u16;let _s=S::<'_>(&n);}",
    ),
    (
        "required-partial",
        false,
        "struct S<T,U>(T,U);fn main(){let _s=S::<u8>(1u8,2u16);}",
    ),
    (
        "const-infer",
        true,
        "struct S<T,const N:usize>([T;N]);fn main(){let s=S([1u8;3]);let _:S<u8,3>=s;}",
    ),
    (
        "const-explicit",
        true,
        "struct S<T,const N:usize>([T;N]);fn main(){let _s=S::<u8,3>([1u8;3]);}",
    ),
    (
        "const-hole",
        true,
        "struct S<T,const N:usize>([T;N]);fn main(){let _s=S::<_,_>([1u8;3]);}",
    ),
    (
        "const-default",
        true,
        "struct S<T,const N:usize=3>([T;N]);fn main(){let _s=S::<u8>([1u8;3]);}",
    ),
    (
        "pattern-explicit",
        true,
        "struct S<T>(T);fn f(s:S<u8>)->u8{let S::<u8>(x)=s;x}",
    ),
    (
        "pattern-hole",
        true,
        "struct S<T>(T);fn f(s:S<u8>)->u8{let S::<_>(x)=s;x}",
    ),
    (
        "bound-missing",
        false,
        "struct N;struct S<T:Copy>(T);fn main(){let _s=S(N);}",
    ),
    (
        "record-generic-body",
        true,
        "struct S<T>{v:T}fn f<T>(x:T)->S<T>{S{v:x}}",
    ),
    (
        "tuple-generic-body",
        true,
        "struct S<T>(T);fn f<T>(x:T)->S<T>{S(x)}",
    ),
    (
        "nested-field-infer",
        true,
        "struct S<T>{v:T}fn main(){let s=S{v:S{v:1u8}};let _:u8=s.v.v;}",
    ),
    (
        "tuple-function-value",
        true,
        "struct S<T>(T);fn main(){let f=S;let _:S<u8>=f(1u8);}",
    ),
    (
        "constructor-fn-pointer",
        true,
        "struct S<T>(T);fn main(){let f:fn(u8)->S<u8>=S;let _s=f(1u8);}",
    ),
    (
        "constructor-return-context",
        true,
        "struct S<T>(T);fn main(){let s:S<u16>=S(1);let _:u16=s.0;}",
    ),
    (
        "record-return-context",
        true,
        "struct S<T>{v:T}fn main(){let s:S<u16>=S{v:1};let _:u16=s.v;}",
    ),
    (
        "distinct-constructions",
        true,
        "struct S<T>(T);fn main(){let a=S(1u8);let b=S(1u16);let _:u8=a.0;let _:u16=b.0;}",
    ),
    (
        "field-shared-variable",
        false,
        "struct S<T>{a:T,b:T}fn main(){let _s=S{a:1u8,b:1u16};}",
    ),
    (
        "const-shared-variable",
        false,
        "struct S<const N:usize>{a:[u8;N],b:[u8;N]}fn main(){let _s=S{a:[1;2],b:[1;3]};}",
    ),
    (
        "const-before-type",
        true,
        "struct S<const N:usize,T>([T;N]);fn main(){let _s=S::<3,u8>([1;3]);}",
    ),
    (
        "const-default-prefix",
        true,
        "struct S<const N:usize,const M:usize=N>([u8;N],[u8;M]);fn main(){let _s=S::<3>([1;3],[2;3]);}",
    ),
    (
        "const-default-prefix-hole",
        true,
        "struct S<const N:usize,const M:usize=N>([u8;N],[u8;M]);fn main(){let _s=S::<_>([1;3],[2;3]);}",
    ),
    (
        "const-default-wrong",
        false,
        "struct S<const N:usize,const M:usize=N>([u8;N],[u8;M]);fn main(){let _s=S::<3>([1;3],[2;4]);}",
    ),
    (
        "constructor-lifetime-type",
        true,
        "struct S<'a,T>(&'a T);fn f<'a,T>(v:&'a T)->S<'a,T>{S::<'a,T>(v)}",
    ),
    (
        "constructor-hrtb",
        true,
        "struct S<T>(T);fn id(x:&u8)->&u8{x}fn main(){let s=S::<for<'a> fn(&'a u8)->&'a u8>(id);let _:for<'b> fn(&'b u8)->&'b u8=s.0;}",
    ),
    (
        "record-update",
        true,
        "struct S<T>{a:T,b:u8}fn main(){let s=S{a:1u16,b:2};let t=S{b:3,..s};let _:u16=t.a;}",
    ),
    (
        "record-explicit",
        true,
        "struct S<T>{v:T}fn main(){let _s=S::<u8>{v:1};}",
    ),
    (
        "record-pattern-explicit",
        true,
        "struct S<T>{v:T}fn f(s:S<u8>)->u8{let S::<u8>{v}=s;v}",
    ),
    (
        "record-pattern-wrong",
        false,
        "struct S<T>{v:T}fn f(s:S<u8>){let S::<u16>{v}=s;}",
    ),
    (
        "enum-type-arguments",
        true,
        "enum E<T>{X(T)}fn main(){let _s=E::<u8>::X(1);}",
    ),
    (
        "enum-variant-arguments",
        true,
        "enum E<T>{X(T)}fn main(){let _s=E::X::<u8>(1);}",
    ),
    (
        "enum-double-arguments",
        false,
        "enum E<T>{X(T)}fn main(){let _s=E::<u8>::X::<u8>(1);}",
    ),
    (
        "enum-record-arguments",
        true,
        "enum E<T>{X{v:T}}fn main(){let _s=E::<u8>::X{v:1};}",
    ),
    (
        "enum-pattern-arguments",
        true,
        "enum E<T>{X(T)}fn f(s:E<u8>)->u8{let E::<u8>::X(x)=s;x}",
    ),
    (
        "alias-record",
        true,
        "struct S<T>{v:T}type A=S<u8>;fn main(){let _s=A{v:1};}",
    ),
    (
        "alias-pattern",
        true,
        "struct S<T>{v:T}type A=S<u8>;fn f(s:A)->u8{let A{v}=s;v}",
    ),
    (
        "alias-enum",
        true,
        "enum E<T>{X(T)}type A=E<u8>;fn main(){let _s=A::X(1);}",
    ),
    (
        "constructor-bound-valid",
        true,
        "struct S<T:Copy>(T);fn f<T:Copy>(v:T)->S<T>{S(v)}",
    ),
    (
        "constructor-bound-invalid",
        false,
        "struct S<T:Copy>(T);fn f<T>(v:T){let _s=S(v);}",
    ),
    (
        "constructor-extra-lifetime",
        false,
        "struct S<T>(T);fn main(){let _s=S::<'static,u8>(1);}",
    ),
    (
        "constructor-missing-lifetime",
        false,
        "struct S<'a,'b,T>(&'a T,&'b T);fn main(){let a=1u8;let _s=S::<'_,u8>(&a,&a);}",
    ),
    (
        "unconstrained-unit",
        false,
        "enum E<T>{X(T),Y}fn main(){let _s=E::Y;}",
    ),
    (
        "unconstrained-constructor",
        false,
        "struct S<T>(T);fn main(){let _f=S;}",
    ),
    (
        "alias-generic-record",
        true,
        "struct S<T>{v:T}type A<U>=S<U>;fn main(){let s=A::<u8>{v:1};let _:u8=s.v;}",
    ),
    (
        "alias-generic-record-hole",
        true,
        "struct S<T>{v:T}type A<U>=S<U>;fn main(){let s=A::<_>{v:1u8};let _:u8=s.v;}",
    ),
    (
        "alias-record-wrong",
        false,
        "struct S<T>{v:T}type A=S<u8>;fn f(s:S<u16>){let A{v}=s;}",
    ),
    (
        "alias-enum-wrong",
        false,
        "enum E<T>{X(T)}type A=E<u8>;fn f(s:E<u16>){let A::X(v)=s;}",
    ),
    (
        "alias-enum-generic",
        true,
        "enum E<T>{X(T)}type A<U>=E<U>;fn main(){let _s=A::<u8>::X(1);}",
    ),
    (
        "alias-enum-generic-hole",
        true,
        "enum E<T>{X(T)}type A<U>=E<U>;fn main(){let _s=A::<_>::X(1u8);}",
    ),
    (
        "alias-enum-variant-args",
        false,
        "enum E<T>{X(T)}type A=E<u8>;fn main(){let _s=A::X::<u8>(1);}",
    ),
    (
        "alias-enum-unit",
        true,
        "enum E<T>{X(T),Y}type A=E<u8>;fn main(){let _s=A::Y;}",
    ),
    (
        "alias-enum-record",
        true,
        "enum E<T>{X{v:T}}type A=E<u8>;fn main(){let _s=A::X{v:1};}",
    ),
    (
        "alias-enum-extra-suffix",
        false,
        "enum E<T>{X(T)}type A=E<u8>;fn main(){let _s=A::X::X(1);}",
    ),
    (
        "alias-enum-module",
        true,
        "mod m{pub enum E<T>{X(T)}pub type A<U>=E<U>;}fn main(){let _s=m::A::<u8>::X(1);}",
    ),
    (
        "alias-enum-import",
        true,
        "enum E<T>{X(T)}type A=E<u8>;use A as B;fn main(){let _s=B::X(1);}",
    ),
];
const ANNOTATIONS: &[(&str, bool, &str)] = &[
    (
        "annotation-type-hole",
        true,
        "struct S<T>(T);fn main(){let s:S<_>=S(1u8);let _:u8=s.0;}",
    ),
    (
        "annotation-const-hole",
        true,
        "struct S<const N:usize>([u8;N]);fn main(){let _s:S<_>=S([1;3]);}",
    ),
    (
        "annotation-lifetime-hole",
        true,
        "struct S<T,U>(T,U);fn f<'a>(x:&'a u8)->S<&'a u8,u8>{let s:S<&'a u8,_>=S(x,1u8);s}",
    ),
    (
        "annotation-alias-hole",
        true,
        "type F<T>=fn(T)->T;fn id(x:u8)->u8{x}fn main(){let f:F<_>=id;let _:u8=f(1u8);}",
    ),
    (
        "annotation-alias-free-binder",
        true,
        "type F<T,U>=fn(T,U)->T;fn id<'b>(x:&'b u8,_:u8)->&'b u8{x}fn f<'a>(x:&'a u8)->&'a u8{let g:F<&'a u8,_>=id;g(x,1u8)}",
    ),
    (
        "annotation-default-free-binder",
        true,
        "struct S<T,const N:usize,F=fn(T)->T>(T,[u8;N],F);fn id(x:&u8)->&u8{x}fn f<'a>(x:&'a u8){let _s:S<&'a u8,_>=S(x,[1u8;3],id);}",
    ),
    (
        "annotation-hrtb-nominal",
        true,
        "struct S<T,U>(T,U);fn g<'a>(x:S<&'a u8,u8>)->&'a u8{x.0}fn main(){let _f:for<'b> fn(S<&'b u8,_>)->&'b u8=g;}",
    ),
    (
        "annotation-hrtb-alias",
        true,
        "type F<T,U>=fn(T,U)->T;fn g<'a>(f:F<&'a u8,u8>,x:&'a u8)->&'a u8{f(x,1u8)}fn main(){let _f:for<'b> fn(F<&'b u8,_>,&'b u8)->&'b u8=g;}",
    ),
    (
        "annotation-hrtb-hole-capture",
        true,
        "struct S<T>(T);fn g<'a>(x:S<&'a u8>,_y:&'a u8){}fn main(){let _f:for<'b> fn(S<_>,&'b u8)=g;}",
    ),
    (
        "annotation-hrtb-nominal-wrong",
        false,
        "struct S<T,U>(T,U);fn g(x:S<&'static u8,u8>)->&'static u8{x.0}fn main(){let _f:for<'b> fn(S<&'b u8,_>)->&'b u8=g;}",
    ),
    (
        "annotation-hrtb-alias-hole",
        false,
        "type F<T>=fn(T)->T;fn g<'a>(f:F<&'a u8>,x:&'a u8)->&'a u8{f(x)}fn main(){let _f:for<'b> fn(F<_>,&'b u8)->&'b u8=g;}",
    ),
    (
        "annotation-hrtb-alias-hole-pair",
        false,
        "type F<T>=(T,fn(T)->T);fn g<'a>(f:F<&'a u8>,x:&'a u8)->&'a u8{(f.1)(x)}fn main(){let _f:for<'b> fn(F<_>,&'b u8)->&'b u8=g;}",
    ),
    (
        "annotation-hrtb-nested-default",
        false,
        "struct S<T,F=fn(T)->T>(T,F);fn g<'a>(s:S<&'a u8>,x:&'a u8)->&'a u8{(s.1)(x)}fn main(){let _f:for<'b> fn(S<_>,&'b u8)->&'b u8=g;}",
    ),
    (
        "annotation-hrtb-alias-hole-static",
        false,
        "type F<T>=(T,fn(T)->T);fn g(f:F<&'static u8>,x:&'static u8)->&'static u8{(f.1)(x)}fn main(){let _f:for<'b> fn(F<_>,&'b u8)->&'b u8=g;}",
    ),
    (
        "constructor-explicit-function-default",
        true,
        "struct S<T,F=fn(T)->T>(T,F);fn id(x:&u8)->&u8{x}fn f<'a>(x:&'a u8)->S<&'a u8>{S::<&'a u8>(x,id)}",
    ),
    (
        "constructor-inferred-function-default",
        true,
        "struct S<T,F=fn(T)->T>(T,F);fn id(x:&u8)->&u8{x}fn f<'a>(x:&'a u8)->S<&'a u8>{S(x,id)}",
    ),
    (
        "annotation-dependent-type-default",
        true,
        "struct S<T,U=T>(T,U);fn main(){let _s:S<_>=S(1u8,2u8);}",
    ),
    (
        "annotation-dependent-type-default-wrong",
        false,
        "struct S<T,U=T>(T,U);fn main(){let _s:S<_>=S(1u8,2u16);}",
    ),
];

fn check(cases: &[(&str, bool, &str)], regions: bool) -> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, accepted, source) in cases {
        rustc::qualify(name, source, accepted)?;
        let result = semantic::analyze(source).and_then(|(_, facts)| {
            if regions {
                let contracts = rustgm::body::regions::collect(&facts)?;
                rustgm::borrow::regions::check(
                    &facts.types,
                    &facts.lifetimes.variances,
                    &contracts,
                )
                .map_err(|e| e.to_string())?;
            }
            Ok(())
        });
        if result.is_ok() != accepted {
            failures.push(format!("{name}: {}", result.err().unwrap_or_default()));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
#[test]
fn constructors_and_patterns_infer_joint_arguments() -> Result<(), Box<dyn std::error::Error>> {
    check(CASES, false)
}
#[test]
fn partial_annotations_preserve_binders_and_prove_lifetime_compatibility()
-> Result<(), Box<dyn std::error::Error>> {
    check(ANNOTATIONS, true)
}
