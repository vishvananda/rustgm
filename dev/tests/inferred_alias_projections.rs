//! Partial alias inputs must be determined independently of their associated outputs.
#[path = "harness/native_references.rs"]
mod native;
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;
const CASES: &[(&str, &str, bool)] = &[
    (
        "inferred-pair",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type P<T>=(T,<T as A>::Out);fn main(){let _:P<_>=(3u16,7u8);}",
        true,
    ),
    (
        "inferred-pair-missing",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}struct N;type P<T>=(T,<T as A>::Out);fn main(){let _:P<_>=(N,7u8);}",
        false,
    ),
    (
        "inferred-pair-wrong-result",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type P<T>=(T,<T as A>::Out);fn main(){let _:P<_>=(3u16,7u32);}",
        false,
    ),
    (
        "inferred-callable-pair",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type F<T>=(T,fn(<T as A>::Out));fn g(_:u8){}fn main(){let _:F<_>=(3u16,g);}",
        true,
    ),
    (
        "inferred-callable-erased",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type F<T>=fn(<T as A>::Out);fn g(_:u8){}fn main(){let _:F<_>=g;}",
        false,
    ),
    (
        "inferred-default",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type P<T,U=<T as A>::Out>=(T,U);fn main(){let _:P<_>=(3u16,7u8);}",
        true,
    ),
    (
        "inferred-default-override",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}struct N;type P<T,U=<T as A>::Out>=(T,U);fn main(){let _:P<_,u32>=(N,7u32);}",
        true,
    ),
    (
        "inferred-erased",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait E{type Out;}impl<T>E for T{type Out=u8;}type P<T>=<<T as A>::Out as E>::Out;fn main(){let _:P<_>=7u8;}",
        false,
    ),
    (
        "reversed-pair",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type P<T>=(<T as A>::Out,T);fn main(){let _:P<_>=(7u8,3u16);}",
        true,
    ),
    (
        "mixed-const",
        "trait A<const N:usize>{type Out;}impl<T:Copy,const N:usize>A<N>for T{type Out=u8;}type P<T,const N:usize>=([T;N],<T as A<N>>::Out);fn main(){let _:P<_,_>=([3u16;2],7u8);}",
        true,
    ),
    (
        "default-wrong",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type P<T,U=<T as A>::Out>=(T,U);fn main(){let _:P<_>=(3u16,7u32);}",
        false,
    ),
    (
        "default-chain",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type P<T,U=<T as A>::Out,V=U>=(T,U,V);fn main(){let _:P<_>=(3u16,7u8,8u8);}",
        true,
    ),
    (
        "source-callable",
        "trait A{type Out;}impl<'a,T:Copy>A for &'a T{type Out=&'a u8;}type P<'a,T>=(T,<&'a T as A>::Out);fn g(x:(u16,&u8)){}fn main(){let _:for<'a>fn(P<'a,_>)=g;}",
        true,
    ),
    (
        "source-callable-static-only",
        "trait A{type Out;}impl<T:Copy>A for &'static T{type Out=&'static u8;}type P<'a,T>=(T,<&'a T as A>::Out);fn g(x:(u16,&u8)){}fn main(){let _:for<'a>fn(P<'a,_>)=g;}",
        false,
    ),
    (
        "source-callable-wrong",
        "trait A{type Out;}impl<'a,T:Copy>A for &'a T{type Out=&'a u8;}type P<'a,T>=(T,<&'a T as A>::Out);fn g(x:(u16,&u16)){}fn main(){let _:for<'a>fn(P<'a,_>)=g;}",
        false,
    ),
    (
        "captured-body",
        "trait A{type Out;}impl<'a>A for &'a u16{type Out=&'a u16;}type P<T>=(T,<T as A>::Out);fn f<'a>(x:&'a u16)->&'a u16{let y:P<_>=(x,x);y.1}fn main(){}",
        true,
    ),
    (
        "captured-body-escape",
        "trait A{type Out;}impl<'a>A for &'a u16{type Out=&'a u16;}type P<T>=(T,<T as A>::Out);fn f<'a>(x:&'a u16)->&'static u16{let y:P<_>=(x,x);y.1}fn main(){}",
        false,
    ),
    (
        "numeric-fallback",
        "trait A{type Out;}impl A for i32{type Out=u8;}type P<T>=(T,<T as A>::Out);fn main(){let _:P<_>=(3,7u8);}",
        true,
    ),
];
#[test]
fn partial_alias_inputs_and_defaults_match_rustc() -> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, source, accepted) in CASES {
        rustc::qualify(&format!("partial-alias-{name}"), source, accepted)?;
        let result = semantic::analyze(source).and_then(|(mut report, mut facts)| {
            let entry =
                rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
            rustgm::lower::instances::build(&report, &mut facts, entry)
                .map(|_| ())
                .map_err(|e| e.to_string())
        });
        if result.is_ok() != accepted {
            failures.push(format!("{name}: {:?}", result.err()));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn inferred_projections_execute_after_input_resolution() -> Result<(), Box<dyn std::error::Error>> {
    native::check_cases(
        "inferred-alias-projections",
        &[
            (
                "reversed",
                "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type P<T>=(<T as A>::Out,T);fn main(){let x:P<_>=(73u8,19u16);check(x.0==73);check(x.1==19)}",
                false,
            ),
            (
                "callable",
                "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type P<T>=(T,fn(<T as A>::Out));fn g(x:u8){check(x==73)}fn main(){let p:P<_>=(19u16,g);check(p.0==19);(p.1)(73)}",
                false,
            ),
            (
                "defaults",
                "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type P<T,U=<T as A>::Out,V=U>=(T,U,V);fn main(){let p:P<_>=(19u16,73u8,51u8);check(p.0==19);check(p.1==73);check(p.2==51)}",
                false,
            ),
            (
                "mixed-const",
                "trait A<const N:usize>{type Out;}impl<T:Copy,const N:usize>A<N>for T{type Out=u8;}type P<T,const N:usize>=([T;N],<T as A<N>>::Out);fn main(){let p:P<_,_>=([19u16;2],73u8);check(p.0[1]==19);check(p.1==73)}",
                false,
            ),
            (
                "source-callable",
                "trait A{type Out;}impl<'a,T:Copy>A for &'a T{type Out=&'a u8;}type P<'a,T>=(T,<&'a T as A>::Out);fn g(x:(u16,&u8)){check(x.0==19);check(*x.1==73)}fn main(){let f:for<'a>fn(P<'a,_>)=g;let x=73;f((19,&x));}",
                false,
            ),
            (
                "captured-body",
                "trait A{type Out;}impl<'a>A for &'a u16{type Out=&'a u16;}type P<T>=(T,<T as A>::Out);fn f<'a>(x:&'a u16)->&'a u16{let p:P<_>=(x,x);p.1}fn main(){let x=73;check(*f(&x)==73)}",
                false,
            ),
        ],
    )
}
