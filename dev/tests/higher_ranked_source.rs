//! Public quantified syntax reaches existing scoped proof and executable consumers.
#[path = "higher_ranked_source/binder_identity.rs"]
mod binder_identity;
#[path = "harness/native_references.rs"]
mod native;
#[path = "helpers/qualify.rs"]
mod rustc;
use std::{
    io::Write,
    process::{Command, Output, Stdio},
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const CASES: &[(&str, &str, bool)] = &[
    (
        "blanket-projection",
        "trait A<'a> { type Out; }\nimpl<'a> A<'a> for u8 { type Out=u8; }\nfn f<T>(x:<T as A<'static>>::Out)-><T as A<'static>>::Out where for<'a>T:A<'a> { x }\nfn main(){if f::<u8>(7)!=7{loop{}}}",
        true,
    ),
    (
        "trait-position-projection",
        "trait A<'a> { type Out; }\nimpl<'a> A<'a> for u8 { type Out=u8; }\nfn f<T:for<'a>A<'a>>(x:<T as A<'static>>::Out)-><T as A<'static>>::Out { x }\nfn main(){if f::<u8>(7)!=7{loop{}}}",
        true,
    ),
    (
        "static-only-rejected",
        "trait A<'a> { type Out; }\nimpl A<'static> for u8 { type Out=u8; }\nfn f<T>(x:<T as A<'static>>::Out)-><T as A<'static>>::Out where for<'a>T:A<'a> { x }\nfn main(){let _=f::<u8>(7);}",
        false,
    ),
    (
        "captured-local-rejected",
        "trait A<'a> { type Out; }\nstruct S<'x>(&'x u8);\nimpl<'a,'x:'a> A<'a> for S<'x>{type Out=&'x u8;}\nfn f<'b,T>(x:<T as A<'b>>::Out)-><T as A<'b>>::Out where for<'a>T:A<'a>{x}\nfn run<'x>(x:&'x u8){let _=f::<S<'x>>(x);}fn main(){let x=7u8;run(&x);}",
        false,
    ),
    (
        "private-witness-local-rejected",
        "trait A<'a> { type Out; }\nstruct S<'x>(&'x u8);\nimpl<'a,'w,'x:'w> A<'a> for S<'x> where 'w:'a {type Out=&'x u8;}\nfn f<'b,T>(x:<T as A<'b>>::Out)-><T as A<'b>>::Out where for<'a>T:A<'a>{x}\nfn run<'x>(x:&'x u8){let _=f::<S<'x>>(x);}fn main(){let x=7u8;run(&x);}",
        false,
    ),
    (
        "nested-callable-projection",
        "trait A<'a> { type Out; }\nimpl<'a>A<'a>for u8{type Out=&'a u8;}\nfn f<T>(x:for<'a>fn(&'a u8)-><T as A<'a>>::Out)->for<'a>fn(&'a u8)-><T as A<'a>>::Out where for<'a>T:A<'a>{x}\nfn id(x:&u8)->&u8{x}\nfn main(){let n=7u8;let g=f::<u8>(id);if *g(&n)!=7{loop{}}}",
        true,
    ),
    (
        "unused-escaping-projection",
        "trait A<'a> { type Out; }\nfn f<'b,T>(x:<T as A<'b>>::Out)->&'static u8 where for<'a>T:A<'a>{let n=7u8;&n}\nfn main(){}",
        false,
    ),
    (
        "copy-parameter",
        "fn f<T>(x:T)->(T,T)where for<'a>T:Copy{(x,x)}fn main(){let p=f(7u8);if p.0!=p.1{loop{}}}",
        true,
    ),
    (
        "clone-only",
        "fn f<T>(x:T)->(T,T)where for<'a>T:Clone{(x,x)}fn main(){}",
        false,
    ),
    (
        "composite-copy",
        "fn f<'x,T>(x:(&'x u8,T))->((&'x u8,T),(&'x u8,T))where for<'a>(&'a u8,T):Copy{(x,x)}fn main(){let v=7;let p=f((&v,3u8));if p.0.1!=p.1.1{loop{}}}",
        true,
    ),
    (
        "composite-no-reverse",
        "fn f<T>(x:T)->(T,T)where for<'a>(&'a u8,T):Copy{(x,x)}fn main(){}",
        false,
    ),
    (
        "two-bounds-one-subject",
        "trait A<'a>{}trait B<'a>{}impl<'a>A<'a>for &'a u8{}impl<'a>B<'a>for &'a u8{}fn f<T>()where for<'a>&'a T:A<'a>+B<'a>{}fn main(){f::<u8>();}",
        true,
    ),
    (
        "reference-subject",
        "trait A<'a>{}impl<'a>A<'a>for &'a u8{}fn f<T>()where for<'a>&'a T:A<'a>{}fn main(){f::<u8>();}",
        true,
    ),
    (
        "fixed-reference-not-universal",
        "trait A<'a>{}impl<'a>A<'a>for &'a u8{}fn f<'x,T>()where &'x T:for<'a>A<'a>{}fn run<'x>(x:&'x u8){f::<'x,u8>();}fn main(){let n=1;run(&n);}",
        false,
    ),
    (
        "trait-argument-capture",
        "trait A<'a,U>{}impl<'a,U>A<'a,U>for u8{}fn f<'x,T:for<'a>A<'a,&'x u8>>(x:&'x u8){}fn main(){let n=1;f::<u8>(&n);}",
        true,
    ),
    (
        "two-independent-locals",
        "trait A<'a,'b>{}impl<'a,'b>A<'a,'b>for u8{}fn f<T:for<'a,'b>A<'a,'b>>(){}fn main(){f::<u8>();}",
        true,
    ),
    (
        "correlated-impl-insufficient",
        "trait A<'a,'b>{}impl<'a>A<'a,'a>for u8{}fn f<T:for<'a,'b>A<'a,'b>>(){}fn main(){f::<u8>();}",
        false,
    ),
    (
        "unused-local",
        "trait A<'a>{}impl<'a>A<'a>for u8{}fn f<T:for<'unused,'a>A<'a>>(){}fn main(){f::<u8>();}",
        true,
    ),
    (
        "nominal-bound",
        "trait A<'a>{}impl<'a>A<'a>for u8{}struct S<T>where for<'a>T:A<'a>{x:T}fn main(){let s=S{x:7u8};if s.x!=7{loop{}}}",
        true,
    ),
    (
        "nominal-static-only",
        "trait A<'a>{}impl A<'static>for u8{}struct S<T>where for<'a>T:A<'a>{x:T}fn main(){let s=S{x:7u8};}",
        false,
    ),
    (
        "implementation-bound",
        "trait A<'a>{}trait B{}impl<'a>A<'a>for u8{}impl<T>B for T where for<'a>T:A<'a>{}fn f<T:B>(){}fn main(){f::<u8>();}",
        true,
    ),
    (
        "implementation-static-only",
        "trait A<'a>{}trait B{}impl A<'static>for u8{}impl<T>B for T where for<'a>T:A<'a>{}fn f<T:B>(){}fn main(){f::<u8>();}",
        false,
    ),
    (
        "supertrait-bound",
        "trait A<'a>{}trait B:for<'a>A<'a>{}impl<'a>A<'a>for u8{}impl B for u8{}fn need<'x,T:A<'x>>(){}fn f<T:B>(){need::<'static,T>();}fn main(){f::<u8>();}",
        true,
    ),
    (
        "supertrait-static-only",
        "trait A<'a>{}trait B:for<'a>A<'a>{}impl A<'static>for u8{}impl B for u8{}fn main(){}",
        false,
    ),
    (
        "outlives-implies-static",
        "fn need<T:'static>(){}fn f<T>()where for<'a>T:'a{need::<T>();}fn main(){f::<u8>();}",
        true,
    ),
    (
        "outlives-rejects-local",
        "fn f<T>(x:T)where for<'a>T:'a{}fn run<'x>(x:&'x u8){f(x);}fn main(){let n=1;run(&n);}",
        false,
    ),
    (
        "where-callable-outlives",
        "fn need<T:'static>(){}fn f<T>()where for<'a>fn(&'a T):'static{need::<T>();}fn main(){f::<u8>();}",
        false,
    ),
    (
        "nested-quantification",
        "trait A<'a,'b>{}fn f<T>()where for<'a>T:for<'b>A<'a,'b>{}fn main(){}",
        false,
    ),
    (
        "trait-position-does-not-bind-subject",
        "trait A<'a>{}fn f<T>()where &'a T:for<'a>A<'a>{}fn main(){}",
        false,
    ),
    (
        "where-callable-outlives-unused",
        "fn need<T:'static>(){}fn f<T>()where for<'a>fn(&'a T):'static{need::<T>();}fn main(){}",
        true,
    ),
    (
        "alpha-renaming",
        "trait A<'a>{}fn f<T>()where for<'a>T:A<'a>,for<'renamed>T:A<'renamed>{}fn main(){}",
        true,
    ),
    (
        "empty-where-quantifier",
        "fn f<T>(x:T)->(T,T)where for<> T:Copy{(x,x)}fn main(){let p=f(7u8);if p.0!=p.1{loop{}}}",
        true,
    ),
    (
        "empty-trait-quantifier",
        "fn f<T:for<> Copy>(x:T)->(T,T){(x,x)}fn main(){let p=f(7u8);if p.0!=p.1{loop{}}}",
        true,
    ),
    (
        "nested-captures",
        "trait A<X>{}fn f<'x,T>(x:&'x T)where for<'a>T:A<for<'b>fn(&'a u8,&'b u8,&'x u8)>{}fn main(){}",
        true,
    ),
    (
        "captured-subject",
        "trait A<'a>{}fn f<'x,T>(x:&'x T)where &'x T:for<'a>A<'a>{}fn main(){}",
        true,
    ),
    (
        "shorthand-local",
        "trait A<'a>{type Out;}fn f<T>()where for<'a>T:A<'a>{let _:T::Out;}fn main(){}",
        true,
    ),
    (
        "shorthand-unused",
        "trait A{type Out;}impl A for u8{type Out=u8;}fn f<T:for<'a>A>(x:T::Out)->T::Out{x}fn main(){if f::<u8>(7)!=7{loop{}}}",
        true,
    ),
    (
        "shorthand-signature",
        "trait A<'a>{type Out;}fn f<T>(x:T::Out)->T::Out where for<'a>T:A<'a>{x}fn main(){}",
        false,
    ),
    (
        "trait-position-lifetime",
        "fn f<T:for<'a>'a>(){}fn main(){}",
        false,
    ),
    (
        "relaxed-binder",
        "fn f<T:?for<'a>Sized>(){}fn main(){}",
        false,
    ),
    (
        "duplicate-local",
        "trait A<'a>{}fn f<T:for<'a,'a>A<'a>>(){}fn main(){}",
        false,
    ),
    (
        "shadow-outer",
        "trait A<'a>{}fn f<'a,T:for<'a>A<'a>>(x:&'a u8){}fn main(){}",
        false,
    ),
    (
        "binder-before-relaxed",
        "fn f<T:for<'a>?Sized>(){}fn main(){}",
        false,
    ),
    (
        "captured-static",
        r"trait A<'a> { type Out; }
struct S<'x>(&'x u8);
impl<'a,'x:'a> A<'a> for S<'x>{type Out=&'x u8;}
fn f<'b,T>(x:<T as A<'b>>::Out)-><T as A<'b>>::Out where for<'a>T:A<'a>{x}
static V:u8=7;fn main(){if *f::<S<'static>>(&V)!=7{loop{}}}",
        true,
    ),
    (
        "private-witness-static",
        r"trait A<'a> { type Out; }
struct S<'x>(&'x u8);
impl<'a,'w,'x:'w> A<'a> for S<'x> where 'w:'a {type Out=&'x u8;}
fn f<'b,T>(x:<T as A<'b>>::Out)-><T as A<'b>>::Out where for<'a>T:A<'a>{x}
static V:u8=7;fn main(){if *f::<S<'static>>(&V)!=7{loop{}}}",
        true,
    ),
    (
        "local-private-intersection",
        r"trait A<'a>{type Out;}struct S<'x>(&'x u8);impl<'a,'w,'x>A<'a>for S<'x> where 'a:'w,'x:'w{type Out=&'x u8;}fn f<'b,T>(x:<T as A<'b>>::Out)-><T as A<'b>>::Out where for<'a>T:A<'a>{x}fn run<'x>(x:&'x u8)->&'x u8{f::<S<'x>>(x)}fn main(){let n=7;if *run(&n)!=7{loop{}}}",
        true,
    ),
    (
        "outer-directed",
        r"trait A<'a>{type Out;}struct S<'x,'y>(&'x u8,&'y u8);impl<'a,'x:'y,'y>A<'a>for S<'x,'y>{type Out=&'x u8;}fn f<'b,T>(x:<T as A<'b>>::Out)-><T as A<'b>>::Out where for<'a>T:A<'a>{x}fn run<'x:'y,'y>(x:&'x u8,y:&'y u8)->&'x u8{f::<S<'x,'y>>(x)}fn main(){let n=7;if *run(&n,&n)!=7{loop{}}}",
        true,
    ),
    (
        "outer-directed-missing",
        r"trait A<'a>{type Out;}struct S<'x,'y>(&'x u8,&'y u8);impl<'a,'x:'y,'y>A<'a>for S<'x,'y>{type Out=&'x u8;}fn f<'b,T>(x:<T as A<'b>>::Out)-><T as A<'b>>::Out where for<'a>T:A<'a>{x}fn run<'x,'y>(x:&'x u8,y:&'y u8)->&'x u8{f::<S<'x,'y>>(x)}fn main(){let n=7;if *run(&n,&n)!=7{loop{}}}",
        false,
    ),
];
fn cli(mode: &str, source: &str) -> std::io::Result<Output> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .arg(mode)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or_else(|| std::io::Error::other("missing source pipe"))?
        .write_all(source.as_bytes())?;
    child.wait_with_output()
}
#[test]
fn quantified_source_proves_lifetimes_and_ownership_before_lowir() -> TestResult {
    for &(name, source, accepted) in CASES {
        rustc::qualify(&format!("quantified-source-{name}"), source, accepted)?;
        let result = cli("--emit-lowir", source)?;
        assert_eq!(
            result.status.code(),
            Some(i32::from(!accepted)),
            "{name}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        if accepted {
            assert!(
                !result.stdout.is_empty() && result.stderr.is_empty(),
                "{name}"
            );
        } else {
            assert!(result.stdout.is_empty(), "{name}");
            let message = std::str::from_utf8(&result.stderr)?;
            assert!(
                !message.contains("scoped consumer")
                    && !message.contains("advanced generic relationships")
                    && !message.contains("instance premise cannot expose")
                    && !message.contains("invalid instance premise lifetime"),
                "{name}: {message}"
            );
        }
    }
    Ok(())
}
#[test]
fn older_inspection_contracts_still_reject_quantified_records() -> TestResult {
    let source = "trait A<'a>{}fn f<T:for<'a>A<'a>>(){}fn main(){}";
    for mode in [
        "--emit-semantics",
        "--count-semantics",
        "--emit-trait-semantics",
        "--count-trait-semantics",
    ] {
        let result = cli(mode, source)?;
        assert_eq!(result.status.code(), Some(1));
        assert!(result.stdout.is_empty());
        assert_eq!(result.stderr, b"ERROR: file 0: 20: higher-ranked trait bounds require advanced generic relationships\n");
    }
    let result = cli("--emit-lowir", source)?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(())
}
#[test]
#[ignore = "requires the pinned rustc and CPPGM native backend"]
fn accepted_quantified_sources_execute_independently() -> TestResult {
    let cases = CASES
        .iter()
        .filter(|case| case.2)
        .map(|&(name, source, _)| (name, source, false))
        .collect::<Vec<_>>();
    native::check_cases("quantified-source", &cases)
}
