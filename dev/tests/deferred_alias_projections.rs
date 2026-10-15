//! Conditional alias type formation is proved at uses without granting written alias bounds.
#[path = "harness/native_references.rs"]
mod native;
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;
const CASES: &[(&str, &str, bool)] = &[
    (
        "generic-unused",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type X<T>=<T as A>::Out;fn main(){}",
        true,
    ),
    (
        "generic-selected",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type X<T>=<T as A>::Out;fn main(){let _:X<u8>=7u8;}",
        true,
    ),
    (
        "generic-missing-use",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}struct N;type X<T>=<T as A>::Out;fn main(){let _:X<N>=7u8;}",
        false,
    ),
    (
        "closed-unused",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}struct N;type X=<N as A>::Out;fn main(){}",
        true,
    ),
    (
        "closed-used",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}struct N;type X=<N as A>::Out;fn main(){let _:X=7u8;}",
        false,
    ),
    (
        "alias-bound-unused",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type X<T:Copy>=<T as A>::Out;fn main(){}",
        true,
    ),
    (
        "alias-bound-no-permission",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}struct N;type X<T:Copy>=<T as A>::Out;fn main(){let _:X<N>=7u8;}",
        false,
    ),
    (
        "symbolic-alias-unused",
        "trait A{type Out;}type X<T:A>=<T as A>::Out;fn main(){}",
        true,
    ),
    (
        "missing-trait-unused",
        "trait A{type Out;}struct N;type X=<N as A>::Out;fn main(){}",
        true,
    ),
    (
        "missing-member-unused",
        "trait A{}type X<T:A>=<T as A>::Out;fn main(){}",
        false,
    ),
    (
        "alias-generic-signature",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type X<T>=<T as A>::Out;fn f<T:Copy>(x:X<T>){}fn main(){}",
        true,
    ),
    (
        "alias-unbounded-signature",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type X<T>=<T as A>::Out;fn f<T>(x:X<T>){}fn main(){}",
        false,
    ),
    (
        "erased-unused",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait E{type Out;}impl<T>E for T{type Out=u16;}type Erase<T>=<T as E>::Out;type X<T>=<T as A>::Out;type Y<T>=Erase<X<T>>;fn main(){}",
        true,
    ),
    (
        "erased-valid",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait E{type Out;}impl<T>E for T{type Out=u16;}type Erase<T>=<T as E>::Out;type X<T>=<T as A>::Out;type Y<T>=Erase<X<T>>;fn main(){let _:Y<u8>=7u16;}",
        true,
    ),
    (
        "erased-invalid",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait E{type Out;}impl<T>E for T{type Out=u16;}type Erase<T>=<T as E>::Out;struct N;type X<T>=<T as A>::Out;type Y<T>=Erase<X<T>>;fn main(){let _:Y<N>=7u16;}",
        false,
    ),
    (
        "zero-array-invalid",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}struct N;type X<T>=[<T as A>::Out;0];fn main(){let _:X<N>=[];}",
        false,
    ),
    (
        "zero-array-valid",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type X<T>=[<T as A>::Out;0];fn main(){let _:X<u8>=[];}",
        true,
    ),
    (
        "alias-default-unused",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type X<T,U=<T as A>::Out>=(T,U);fn main(){}",
        true,
    ),
    (
        "alias-default-valid",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type X<T,U=<T as A>::Out>=(T,U);fn main(){let _:X<u16>=(2u16,7u8);}",
        true,
    ),
    (
        "alias-default-invalid",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}struct N;type X<T,U=<T as A>::Out>=(T,U);fn main(){let _:X<N>=(N,7u8);}",
        false,
    ),
    (
        "alias-default-overridden",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}struct N;type X<T,U=<T as A>::Out>=(T,U);fn main(){let _:X<N,u16>=(N,7u16);}",
        true,
    ),
    (
        "nominal-default-unused",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}struct X<T,U=<T as A>::Out>(T,U);fn main(){}",
        true,
    ),
    (
        "trait-default-unused",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait X<T,U=<T as A>::Out>{}fn main(){}",
        true,
    ),
    (
        "closed-signature",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}struct N;type X=<N as A>::Out;fn f(x:X){}fn main(){}",
        false,
    ),
    (
        "alias-cycle",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type X=<Y as A>::Out;type Y=X;fn main(){}",
        false,
    ),
    (
        "associated-definition-must-prove",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B{type Value;}impl<T>B for T{type Value=<T as A>::Out;}fn main(){}",
        false,
    ),
    (
        "const-alias-unused",
        "trait A<const N:usize>{type Out;}impl A<3>for u8{type Out=u16;}type X<const N:usize>=<u8 as A<N>>::Out;fn main(){}",
        true,
    ),
    (
        "const-alias-selected",
        "trait A<const N:usize>{type Out;}impl A<3>for u8{type Out=u16;}type X<const N:usize>=<u8 as A<N>>::Out;fn main(){let _:X<3>=7u16;}",
        true,
    ),
    (
        "const-alias-invalid",
        "trait A<const N:usize>{type Out;}impl A<3>for u8{type Out=u16;}type X<const N:usize>=<u8 as A<N>>::Out;fn main(){let _:X<4>=7u16;}",
        false,
    ),
    (
        "nested-unused",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type X<T>=fn(<T as A>::Out);fn main(){}",
        true,
    ),
    (
        "nominal-default-selected-valid",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}struct N;struct X<T,U=<T as A>::Out>(T,U);fn main(){let _:X<u16>=X(1u16,7u8);}",
        true,
    ),
    (
        "nominal-default-selected-invalid",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}struct N;struct X<T,U=<T as A>::Out>(T,U);fn main(){let _:X<N>=X(N,7u8);}",
        false,
    ),
    (
        "nominal-default-override",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}struct N;struct X<T,U=<T as A>::Out>(T,U);fn main(){let _:X<N,u16>=X(N,7u16);}",
        true,
    ),
    (
        "nominal-closed-default-unused",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}struct N;struct X<U=<N as A>::Out>(U);fn main(){}",
        false,
    ),
    (
        "alias-closed-default-unused",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}struct N;type X<U=<N as A>::Out>=U;fn main(){}",
        true,
    ),
    (
        "trait-closed-default-unused",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}struct N;trait X<U=<N as A>::Out>{}fn main(){}",
        false,
    ),
    (
        "derived-subject-valid",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}#[derive(Clone,Copy)]struct S(u8);type X=<S as A>::Out;fn main(){let _:X=7u8;}",
        true,
    ),
    (
        "invalid-derived-subject",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}struct N;#[derive(Clone,Copy)]struct S(N);type X=<S as A>::Out;fn main(){}",
        false,
    ),
    (
        "generic-derived-alias-unused",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}struct N;#[derive(Clone,Copy)]struct S<T>(T);type X=<S<N> as A>::Out;fn main(){}",
        true,
    ),
    (
        "generic-derived-alias-used",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}struct N;#[derive(Clone,Copy)]struct S<T>(T);type X=<S<N> as A>::Out;fn main(){let _:X=7u8;}",
        false,
    ),
    (
        "erased-bound-unused",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait E{type Out;}impl<T>E for T{type Out=u16;}type Erase<T>=<T as E>::Out;type X<T>=Erase<<T as A>::Out>;trait B<U>{}fn f<T:Copy>()where u8:B<X<T>>{}fn main(){}",
        true,
    ),
    (
        "erased-bound-called",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait E{type Out;}impl<T>E for T{type Out=u16;}type Erase<T>=<T as E>::Out;type X<T>=Erase<<T as A>::Out>;trait B<U>{}fn f<T:Copy>()where u8:B<X<T>>{}fn main(){f::<u8>();}",
        false,
    ),
    (
        "erased-subject-unused",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait E{type Out;}impl<T>E for T{type Out=u16;}type Erase<T>=<T as E>::Out;type X<T>=Erase<<T as A>::Out>;trait B<U>{}trait C{}fn f<T:Copy>()where X<T>:C{}fn main(){}",
        true,
    ),
    (
        "erased-alias-early-lifetime",
        "trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=u8;}trait E{type Out;}impl<T>E for T{type Out=u16;}type Erase<T>=<T as E>::Out;type X<'a>=Erase<<u8 as A<'a>>::Out>;fn f<'a>()where X<'a>:Copy{}fn main(){f::<'static>();}",
        true,
    ),
    (
        "erased-alias-copy-unused",
        "trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=u8;}trait E{type Out;}struct N;impl<T>E for T{type Out=N;}type Erase<T>=<T as E>::Out;type X<'a>=Erase<<u8 as A<'a>>::Out>;fn f<'a>()where X<'a>:Copy{}fn main(){}",
        true,
    ),
    (
        "erased-alias-copy-called",
        "trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=u8;}trait E{type Out;}struct N;impl<T>E for T{type Out=N;}type Erase<T>=<T as E>::Out;type X<'a>=Erase<<u8 as A<'a>>::Out>;fn f<'a>()where X<'a>:Copy{}fn main(){f::<'static>();}",
        false,
    ),
    (
        "implicit-sized-alias",
        "trait A{type Out;}impl<T>A for T{type Out=u8;}type X<T>=<T as A>::Out;fn main(){let _:X<str>=7u8;}",
        false,
    ),
    (
        "explicit-sized-alias",
        "trait A{type Out;}impl<T>A for T{type Out=u8;}type X<T:Sized>=<T as A>::Out;fn main(){let _:X<str>=7u8;}",
        false,
    ),
    (
        "relaxed-sized-alias",
        "trait A{type Out;}impl<T>A for T{type Out=u8;}type X<T:?Sized>=<T as A>::Out;fn main(){}",
        true,
    ),
    (
        "unsized-blanket-alias",
        "trait A{type Out;}impl<T:?Sized>A for T{type Out=u8;}type X<T>=<T as A>::Out;fn main(){let _:X<str>=7u8;}",
        true,
    ),
    (
        "inferred-pair",
        "trait A{type Out;}impl<T>A for T{type Out=u8;}type P<T>=(T,<T as A>::Out);fn main(){let _:P<_>=(1u16,2u8);}",
        true,
    ),
    (
        "inferred-noninjective",
        "trait A{type Out;}impl<T>A for T{type Out=u8;}type X<T>=<T as A>::Out;fn main(){let _:X<_>=7u8;}",
        false,
    ),
    (
        "inferred-pair-unsized",
        "trait A{type Out;}impl<T>A for T{type Out=u8;}type P<T>=(*const T,<T as A>::Out);fn f(x:*const str){let _:P<_>=(x,2u8);}fn main(){}",
        false,
    ),
    (
        "inferred-nested-pair",
        "trait A{type Out;}impl<T>A for T{type Out=u8;}type F<T>=(T,fn(<T as A>::Out));fn g(_:u8){}fn main(){let _:F<_>=(1u16,g);}",
        true,
    ),
    (
        "inferred-default-pair",
        "trait A{type Out;}impl<T>A for T{type Out=u8;}type P<T,U=<T as A>::Out>=(T,U);fn main(){let _:P<_>=(1u16,2u8);}",
        true,
    ),
    (
        "inferred-default-unsized",
        "trait A{type Out;}impl<T>A for T{type Out=u8;}type P<T,U=<T as A>::Out>=(*const T,U);fn f(x:*const str){let _:P<_>=(x,2u8);}fn main(){}",
        false,
    ),
    (
        "inferred-default-overridden",
        "trait A{type Out;}impl<T>A for T{type Out=u8;}type P<T,U=<T as A>::Out>=(*const T,U);fn f(x:*const str){let _:P<_,u8>=(x,2u8);}fn main(){}",
        true,
    ),
    (
        "inferred-alias-chain",
        "trait A{type Out;}impl<T>A for T{type Out=u8;}type X<T>=<T as A>::Out;type Y<T>=X<T>;type P<T>=(T,Y<T>);fn main(){let _:P<_>=(1u16,2u8);}",
        true,
    ),
    (
        "recursive-sized-premise",
        "trait B{}impl<T>B for T{}trait A{type Out;}impl<T:?Sized+B>A for *const T{type Out=u8;}type X<T>=<*const T as A>::Out;fn main(){let _:X<str>=7u8;}",
        false,
    ),
    (
        "recursive-sized-positive",
        "trait B{}impl<T>B for T{}trait A{type Out;}impl<T:?Sized+B>A for *const T{type Out=u8;}type X<T>=<*const T as A>::Out;fn main(){let _:X<u8>=7u8;}",
        true,
    ),
    (
        "sized-caller-bound",
        "trait A{type Out;}impl<T>A for T{type Out=u8;}type X<T>=<T as A>::Out;fn f<T:?Sized>()where T:Sized{let _:X<T>=7u8;}fn main(){}",
        true,
    ),
    (
        "sized-caller-missing",
        "trait A{type Out;}impl<T>A for T{type Out=u8;}type X<T>=<T as A>::Out;fn f<T:?Sized>(){let _:X<T>=7u8;}fn main(){}",
        false,
    ),
];
#[test]
fn deferred_alias_requirements_match_independent_use_checks()
-> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, source, accepted) in CASES {
        rustc::qualify(&format!("deferred-alias-{name}"), source, accepted)?;
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
            let required = match name {
                "missing-member-unused" => "qualified trait has no prepared associated type",
                "alias-cycle" => "recursive type alias",
                "invalid-derived-subject" => "requires every field to implement",
                "implicit-sized-alias"
                | "explicit-sized-alias"
                | "recursive-sized-premise"
                | "sized-caller-missing" => "alias projection requires a Sized type argument",
                "inferred-noninjective" => "type annotations needed",
                "inferred-pair-unsized" | "inferred-default-unsized" => {
                    "generic argument must be sized"
                }
                "erased-alias-copy-called" => "must implement Copy",
                "erased-bound-called" => {
                    "trait obligation requires an unambiguous matching assumption or implementation"
                }
                _ => "associated type requires an unambiguous implementation",
            };
            if result
                .as_ref()
                .err()
                .is_none_or(|error| !error.contains(required))
            {
                failures.push(format!("{name}: wrong rejection: {:?}", result.err()));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn deferred_aliases_execute_after_use_proofs() -> Result<(), Box<dyn std::error::Error>> {
    native::check_cases(
        "deferred-alias-projections",
        &[
            (
                "inferred-pair",
                "trait A{type Out;}impl<T>A for T{type Out=u8;}type P<T>=(T,<T as A>::Out);fn main(){let x:P<_>=(73u16,2u8);check(x.0==73u16);check(x.1==2u8)}",
                false,
            ),
            (
                "inferred-nested-pair",
                "trait A{type Out;}impl<T>A for T{type Out=u8;}type F<T>=(T,fn(<T as A>::Out));fn g(x:u8){check(x==73)}fn main(){let x:F<_>=(1u16,g);(x.1)(73)}",
                false,
            ),
            (
                "generic-signature",
                "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type X<T>=<T as A>::Out;fn f<T:Copy>()->X<T>{73}fn main(){check(f::<u16>()==73)}",
                false,
            ),
            (
                "erased-result",
                "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait E{type Out;}impl<T>E for T{type Out=u16;}type Erase<T>=<T as E>::Out;type X<T>=<T as A>::Out;type Y<T>=Erase<X<T>>;fn main(){let x:Y<u8>=73;check(x==73u16)}",
                false,
            ),
            (
                "overridden-default",
                "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}struct N;type X<T,U=<T as A>::Out>=(T,U);fn main(){let x:X<N,u16>=(N,73);check(x.1==73u16)}",
                false,
            ),
            (
                "selected-nominal-default",
                "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}struct X<T,U=<T as A>::Out>(T,U);fn main(){let x:X<u16>=X(1u16,73);check(x.1==73u8)}",
                false,
            ),
            (
                "const-alias-selection",
                "trait A<const N:usize>{type Out;}impl A<3>for u8{type Out=u16;}type X<const N:usize>=<u8 as A<N>>::Out;fn main(){let x:X<3>=73;check(x==73u16)}",
                false,
            ),
        ],
    )
}
