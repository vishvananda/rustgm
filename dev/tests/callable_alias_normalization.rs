//! Callable projection proofs preserve local universals, captured callers and body allocation.
use crate::native_references as native;
use crate::qualify as rustc;
use crate::semantic;
const CASES: &[(&str, &str, bool)] = &[
    (
        "callable-use",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type F<T>=fn(<T as A>::Out);fn f(_:F<u8>){}fn main(){}",
        true,
    ),
    (
        "callable-use-missing",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}struct N;type F<T>=fn(<T as A>::Out);fn f(_:F<N>){}fn main(){}",
        false,
    ),
    (
        "callable-caller",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type F<T>=fn(<T as A>::Out);fn f<T:Copy>(_:F<T>){}fn main(){}",
        true,
    ),
    (
        "callable-caller-missing",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type F<T>=fn(<T as A>::Out);fn f<T>(_:F<T>){}fn main(){}",
        false,
    ),
    (
        "quantified-callable-capture",
        "trait A<'a>{type Out;}impl<'a,T:Copy+'a>A<'a>for T{type Out=&'a T;}type F<T>=for<'a>fn(&'a T,<T as A<'a>>::Out);fn f(_:F<u8>){}fn main(){}",
        true,
    ),
    (
        "quantified-callable-short-input",
        "trait A<'a>{type Out;}impl<'a,T:Copy+'a>A<'a>for T{type Out=&'a T;}type F<T>=for<'a>fn(&'a T,<T as A<'a>>::Out);fn f<'b>(_:F<&'b u8>){}fn main(){}",
        false,
    ),
    (
        "quantified-callable-short-result",
        "trait A<'a>{type Out;}impl<'a,T:Copy+'a>A<'a>for T{type Out=&'a T;}type F<T>=for<'a>fn()-><T as A<'a>>::Out;fn f<'b>(_:F<&'b u8>){}fn main(){}",
        false,
    ),
    (
        "callable-local",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type F<T>=fn(<T as A>::Out);fn g(_:u8){}fn main(){let _:F<u8>=g;}",
        true,
    ),
    (
        "callable-local-missing",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type F<T>=fn(<T as A>::Out);struct N;fn g(_:u8){}fn main(){let _:F<N>=g;}",
        false,
    ),
    (
        "callable-erased",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type F<T>=fn(<T as A>::Out);trait E{type Out;}impl<T>E for T{type Out=u16;}type X<T>=<F<T> as E>::Out;fn main(){let _:X<u8>=7u16;}",
        true,
    ),
    (
        "callable-erased-missing",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type F<T>=fn(<T as A>::Out);struct N;trait E{type Out;}impl<T>E for T{type Out=u16;}type X<T>=<F<T> as E>::Out;fn main(){let _:X<N>=7u16;}",
        false,
    ),
    (
        "callable-default",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type F<T>=fn(<T as A>::Out);type X<T,U=F<T>>=(T,U);fn g(_:u8){}fn main(){let _:X<u8>=(7u8,g);}",
        true,
    ),
    (
        "callable-default-missing",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type F<T>=fn(<T as A>::Out);struct N;type X<T,U=F<T>>=(T,U);fn g(_:u8){}fn main(){let _:X<N>=(N,g);}",
        false,
    ),
    (
        "callable-default-override",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type F<T>=fn(<T as A>::Out);struct N;type X<T,U=F<T>>=(T,U);fn main(){let _:X<N,u8>=(N,7u8);}",
        true,
    ),
    (
        "quantified-captured-static",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a,T:Copy+'b>A<'a,'b>for T{type Out=&'b T;}type P<'a,'b,T>=<T as A<'a,'b>>::Out;type F<'a,T>=for<'b>fn(P<'b,'a,T>);fn f(_:F<'static,u8>){}fn main(){}",
        true,
    ),
    (
        "quantified-captured-short",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a,T:Copy+'b>A<'a,'b>for T{type Out=&'b T;}type P<'a,'b,T>=<T as A<'a,'b>>::Out;type F<'a,T>=for<'b>fn(P<'b,'a,T>);fn f<'a>(_:F<'a,u8>){}fn main(){}",
        false,
    ),
    (
        "quantified-local-cannot-static",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a,T:Copy+'b>A<'a,'b>for T{type Out=&'b T;}type P<'a,'b,T>=<T as A<'a,'b>>::Out;type F<T>=for<'b>fn(P<'static,'b,T>);fn f(_:F<u8>){}fn main(){}",
        false,
    ),
    (
        "quantified-local-equality",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a,T:Copy+'b>A<'a,'b>for T{type Out=&'b T;}type P<'a,'b,T>=<T as A<'a,'b>>::Out;type F<T>=for<'b>fn(P<'b,'b,T>);fn f(_:F<u8>){}fn main(){}",
        true,
    ),
    (
        "nested-captured-binder",
        "trait A<'a>{type Out;}impl<'a,T:Copy+'a>A<'a>for T{type Out=&'a T;}type P<'a,T>=<T as A<'a>>::Out;type F<T>=for<'a>fn(&'a T,for<'b>fn(&'b T,P<'a,T>));fn f(_:F<u8>){}fn main(){}",
        true,
    ),
    (
        "nested-generic-static",
        "trait A<'a>{type Out;}impl<'a,T:Copy+'a>A<'a>for T{type Out=&'a T;}type P<'a,T>=<T as A<'a>>::Out;type F<T>=for<'a>fn(&'a T,for<'b>fn(&'b T,P<'a,T>));fn f<T:Copy+'static>(_:F<T>){}fn main(){}",
        true,
    ),
    (
        "sibling-callables",
        "trait A<'a>{type Out;}impl<'a,T:Copy+'a>A<'a>for T{type Out=&'a T;}type P<'a,T>=<T as A<'a>>::Out;type F<T>=(for<'a>fn(&'a T,P<'a,T>),for<'a,'b>fn(&'a T,&'b T,P<'a,T>));fn f(_:F<u8>){}fn main(){}",
        true,
    ),
    (
        "sibling-captured-short",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a,T:Copy+'b>A<'a,'b>for T{type Out=&'b T;}type P<'a,'b,T>=<T as A<'a,'b>>::Out;type F<'a,T>=(for<'b>fn(P<'b,'b,T>),for<'b>fn(P<'b,'a,T>));fn f<'a>(_:F<'a,u8>){}fn main(){}",
        false,
    ),
    (
        "sibling-captured-static",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a,T:Copy+'b>A<'a,'b>for T{type Out=&'b T;}type P<'a,'b,T>=<T as A<'a,'b>>::Out;type F<'a,T>=(for<'b>fn(P<'b,'b,T>),for<'b>fn(P<'b,'a,T>));fn f(_:F<'static,u8>){}fn main(){}",
        true,
    ),
    (
        "body-static",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a,T:Copy+'b>A<'a,'b>for T{type Out=&'b T;}type P<'a,'b,T>=<T as A<'a,'b>>::Out;type F<'a,T>=for<'b>fn(P<'b,'a,T>);fn g(_: &u8){}fn main(){let _:F<'static,u8>=g;}",
        true,
    ),
    (
        "body-short",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a,T:Copy+'b>A<'a,'b>for T{type Out=&'b T;}type P<'a,'b,T>=<T as A<'a,'b>>::Out;type F<'a,T>=for<'b>fn(P<'b,'a,T>);fn g(_: &u8){}fn f<'a>(_:&'a u8){let _:F<'a,u8>=g;}fn main(){}",
        false,
    ),
    (
        "body-mixed-static",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a,T:Copy+'b>A<'a,'b>for T{type Out=&'b T;}type P<'a,'b,T>=<T as A<'a,'b>>::Out;type F<'a,T>=for<'b>fn(P<'b,'a,T>);fn g(_: &u8){}fn f(){let _:F<'static,u8>=g;let _:F<'static,u8>=g;}fn main(){f();}",
        true,
    ),
    (
        "body-mixed-short",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a,T:Copy+'b>A<'a,'b>for T{type Out=&'b T;}type P<'a,'b,T>=<T as A<'a,'b>>::Out;type F<'a,T>=for<'b>fn(P<'b,'a,T>);fn g(_: &u8){}fn f<'a>(_:&'a u8){let _:F<'static,u8>=g;let _:F<'a,u8>=g;}fn main(){}",
        false,
    ),
    (
        "independent-bodies-short",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a,T:Copy+'b>A<'a,'b>for T{type Out=&'b T;}type P<'a,'b,T>=<T as A<'a,'b>>::Out;type F<'a,T>=for<'b>fn(P<'b,'a,T>);fn g(_: &u8){}fn good(){let _:F<'static,u8>=g;}fn bad<'a>(_:&'a u8){let _:F<'a,u8>=g;}fn main(){}",
        false,
    ),
    (
        "interleaved-bodies-static",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a,T:Copy+'b>A<'a,'b>for T{type Out=&'b T;}type P<'a,'b,T>=<T as A<'a,'b>>::Out;type F<'a,T>=for<'b>fn(P<'b,'a,T>);fn g(_: &u8){}fn f(){let _:F<'static,u8>=g;fn h(){let _:F<'static,u8>=g;}let _:F<'static,u8>=g;h();}fn main(){f();}",
        true,
    ),
];
#[test]
fn callable_alias_requirements_match_independent_scope_checks()
-> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, source, accepted) in CASES {
        rustc::qualify(&format!("callable-alias-{name}"), source, accepted)?;
        let result = semantic::analyze(source).and_then(|(mut report, mut facts)| {
            let entry =
                rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
            rustgm::lower::instances::build(&report, &mut facts, entry)
                .map(|_| ())
                .map_err(|e| e.to_string())
        });
        if result.is_ok() != accepted {
            failures.push(format!("{name}: {:?}", result.err()));
        } else if !accepted {
            let required = if name.ends_with("missing") {
                "associated type requires an unambiguous implementation"
            } else if name == "quantified-callable-short-result" {
                "function pointer output lifetime is unconstrained by its inputs"
            } else {
                "required lifetime does not outlive its target"
            };
            if result.as_ref().err().is_none_or(|e| !e.contains(required)) {
                failures.push(format!("{name}: wrong rejection: {:?}", result.err()));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn callable_aliases_execute_with_scoped_projection_evidence()
-> Result<(), Box<dyn std::error::Error>> {
    native::check_cases(
        "callable-alias-normalization",
        &[
            (
                "local-pointer",
                "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type F<T>=fn(<T as A>::Out);fn g(x:u8){check(x==73)}fn main(){let f:F<u8>=g;f(73)}",
                false,
            ),
            (
                "generic-signature",
                "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type F<T>=fn(<T as A>::Out);fn f<T:Copy>(x:F<T>)->F<T>{x}fn g(x:u8){check(x==73)}fn main(){let g=f::<u16>(g);g(73)}",
                false,
            ),
            (
                "erased-result",
                "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type F<T>=fn(<T as A>::Out);trait E{type Out;}impl<T>E for T{type Out=u16;}type X<T>=<F<T> as E>::Out;fn main(){let x:X<u8>=73;check(x==73u16)}",
                false,
            ),
            (
                "default",
                "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type F<T>=fn(<T as A>::Out);type X<T,U=F<T>>=(T,U);fn g(x:u8){check(x==73)}fn main(){let x:X<u8>=(73,g);(x.1)(x.0)}",
                false,
            ),
            (
                "captured-static",
                "trait A<'a,'b>{type Out;}impl<'a,'b:'a,T:Copy+'b>A<'a,'b>for T{type Out=&'b T;}type P<'a,'b,T>=<T as A<'a,'b>>::Out;type F<'a,T>=for<'b>fn(P<'b,'a,T>);fn g(x:&u8){check(*x==73)}fn main(){let f:F<'static,u8>=g;f(&73)}",
                false,
            ),
            (
                "nested-callable",
                "trait A<'a>{type Out;}impl<'a,T:Copy+'a>A<'a>for T{type Out=&'a T;}type P<'a,T>=<T as A<'a>>::Out;type F<T>=for<'a>fn(&'a T,for<'b>fn(&'b T,P<'a,T>));fn apply<'a>(x:&'a u8,read:for<'b>fn(&'b u8,&'a u8)){read(&1,x)}fn read(_: &u8,y:&u8){check(*y==73)}fn main(){let f:F<u8>=apply;f(&73,read)}",
                false,
            ),
        ],
    )
}
