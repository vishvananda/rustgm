//! Signature projections select with completed caller assumptions and preserve inference inputs.
use crate::native_references as native;
use crate::qualify as rustc;
use crate::semantic;
const CASES: &[(&str, &str, bool)] = &[
    (
        "signature-input",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}fn f<T:Copy>(x:<T as A>::Out){}fn main(){}",
        true,
    ),
    (
        "signature-output",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}fn f<T:Copy>()-><T as A>::Out{7}fn main(){}",
        true,
    ),
    (
        "where-bound",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}fn f<T>(x:<T as A>::Out)where T:Copy{}fn main(){}",
        true,
    ),
    (
        "missing-signature-bound",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}fn f<T>(x:<T as A>::Out){}fn main(){}",
        false,
    ),
    (
        "local-control",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}fn f<T:Copy>(){let x:<T as A>::Out=7;}fn main(){}",
        true,
    ),
    (
        "clone-is-insufficient",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}fn f<T:Clone>()-><T as A>::Out{7}fn main(){}",
        false,
    ),
    (
        "multiple-projections",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}fn f<T:Copy>(a:<T as A>::Out,b:<T as A>::Out)-><T as A>::Out{a+b}fn main(){}",
        true,
    ),
    (
        "independent-owner-missing",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}fn f<T:Copy>()-><T as A>::Out{7}fn g<T>()-><T as A>::Out{8}fn main(){}",
        false,
    ),
    (
        "output-with-later-lifetime",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}fn f<'a,T:Copy>(x:<T as A>::Out,y:&'a u8)->&'a u8{y}fn main(){}",
        true,
    ),
    (
        "lifetime-where-bound",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}fn f<'a,T>(x:<T as A>::Out,y:&'a T)where T:Copy+'a{}fn main(){}",
        true,
    ),
    (
        "nested-callable",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}fn f<T:Copy>(x:for<'a>fn(&'a u8,<T as A>::Out)->u8){}fn main(){}",
        true,
    ),
    (
        "trait-method-inherits",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait M<T:Copy>{fn f(x:<T as A>::Out);}fn main(){}",
        true,
    ),
    (
        "inherent-method-inherits",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}struct S<T>(T);impl<T:Copy>S<T>{fn f(x:<T as A>::Out){}}fn main(){}",
        true,
    ),
    (
        "user-trait-evidence",
        "trait B{}trait A{type Out;}impl<T:B>A for T{type Out=u8;}fn f<T:B>()-><T as A>::Out{7}fn main(){}",
        true,
    ),
    (
        "user-trait-missing",
        "trait B{}trait A{type Out;}impl<T:B>A for T{type Out=u8;}fn f<T>()-><T as A>::Out{7}fn main(){}",
        false,
    ),
    (
        "supertrait-evidence",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B:Copy{}fn f<T:B>()-><T as A>::Out{7}fn main(){}",
        true,
    ),
    (
        "complete-argument-environment",
        "trait B<U>{}trait A<U>{type Out;}impl<T:B<U>,U>A<U>for T{type Out=u8;}fn f<T:B<U>,U>()-><T as A<U>>::Out{7}fn main(){}",
        true,
    ),
    (
        "does-not-infer-projection-input",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}fn f<T:Copy>(x:<T as A>::Out){}fn main(){f(7u8)}",
        false,
    ),
    (
        "explicit-projection-input",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}fn f<T:Copy>(x:<T as A>::Out){}fn main(){f::<u8>(7u8)}",
        true,
    ),
    (
        "closed-projection-bound-control",
        "trait A{type Out;}impl A for u8{type Out=u8;}fn f()where <u8 as A>::Out:Copy{}fn main(){}",
        true,
    ),
    (
        "projected-bound-after",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}fn f<T>()where T:Copy,<T as A>::Out:Copy{}fn main(){}",
        true,
    ),
    (
        "projected-bound-before",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}fn f<T>()where <T as A>::Out:Copy,T:Copy{}fn main(){}",
        true,
    ),
    (
        "projected-bound-input",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}fn f<T>(x:<T as A>::Out)where <T as A>::Out:Copy,T:Copy{}fn main(){}",
        true,
    ),
    (
        "projected-bound-output",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}fn f<T>()-><T as A>::Out where <T as A>::Out:Copy,T:Copy{7}fn main(){}",
        true,
    ),
    (
        "composite-subject",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}fn f<T>()where (<T as A>::Out,u8):Copy,T:Copy{}fn main(){}",
        true,
    ),
    (
        "reference-subject",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}fn f<'a,T>()where &'a <T as A>::Out:Copy,T:Copy{}fn main(){}",
        true,
    ),
    (
        "repeated-subject",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}fn f<T>()where <T as A>::Out:Copy+Clone,T:Copy{}fn main(){}",
        true,
    ),
    (
        "bound-does-not-prove-selection",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}fn f<T>()where <T as A>::Out:Copy{}fn main(){}",
        false,
    ),
    (
        "bounded-trait-result",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B{}impl B for u8{}fn f<T>()where <T as A>::Out:B,T:Copy{}fn main(){}",
        true,
    ),
    (
        "deferred-generic-result-bound",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B{}fn f<T>()where <T as A>::Out:B,T:Copy{}fn main(){}",
        true,
    ),
    (
        "closed-projection-missing-copy",
        "trait A{type Out;}struct S;impl A for u8{type Out=S;}fn f()where <u8 as A>::Out:Copy{}fn main(){}",
        false,
    ),
    (
        "recursive-associated-result",
        "trait A{type Out;}impl A for u8{type Out=<u8 as A>::Out;}fn f()where <u8 as A>::Out:Copy{}fn main(){}",
        false,
    ),
    (
        "nominal-default-unused",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}struct S<T:Copy,U=<T as A>::Out>(T,U);fn main(){}",
        true,
    ),
    (
        "nominal-default-selected",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}struct S<T:Copy,U=<T as A>::Out>(T,U);fn main(){let _:S<u8>=S(1u8,2u8);}",
        true,
    ),
    (
        "nominal-default-overridden",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}struct S<T:Copy,U=<T as A>::Out>(T,U);fn main(){let _:S<u8,u16>=S(1u8,2u16);}",
        true,
    ),
    (
        "nominal-default-owner-bound-required",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}struct S<T:Copy,U=<T as A>::Out>(T,U);struct N;fn main(){let _:S<N,u16>=S(N,2u16);}",
        false,
    ),
    (
        "enum-default-selected",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}enum E<T:Copy,U=<T as A>::Out>{V(T,U)}fn main(){let _:E<u8>=E::V(1u8,2u8);}",
        true,
    ),
    (
        "generic-result-bound-invoked",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B{}fn f<T>()where <T as A>::Out:B,T:Copy{}fn main(){f::<u8>();}",
        false,
    ),
    (
        "written-closed-result-bound",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B{}fn f<T:Copy>()where <u8 as A>::Out:B{}fn main(){}",
        false,
    ),
    (
        "lifetime-only-assumption",
        "trait B<'a>{}trait A<'a>{type Out;}impl<'a>A<'a>for u8 where u8:B<'a>{type Out=u8;}fn f<'a>()-><u8 as A<'a>>::Out where u8:B<'a>{7}fn main(){}",
        true,
    ),
    (
        "lifetime-only-missing",
        "trait B<'a>{}trait A<'a>{type Out;}impl<'a>A<'a>for u8 where u8:B<'a>{type Out=u8;}fn f<'a>()-><u8 as A<'a>>::Out{7}fn main(){}",
        false,
    ),
    (
        "lifetime-only-invoked",
        "trait B<'a>{}trait A<'a>{type Out;}impl<'a>A<'a>for u8 where u8:B<'a>{type Out=u8;}fn f<'a>()-><u8 as A<'a>>::Out where u8:B<'a>{7}fn main(){f::<'static>();}",
        false,
    ),
    (
        "lifetime-copy-unused",
        "trait A<'a>{type Out;}struct N;impl<'a>A<'a>for u8{type Out=N;}fn f<'a>()where <u8 as A<'a>>::Out:Copy{}fn main(){}",
        true,
    ),
    (
        "lifetime-copy-called",
        "trait A<'a>{type Out;}struct N;impl<'a>A<'a>for u8{type Out=N;}fn f<'a>()where <u8 as A<'a>>::Out:Copy{}fn main(){f::<'static>();}",
        false,
    ),
    (
        "type-copy-unused",
        "trait A{type Out;}struct N;impl<T>A for T{type Out=N;}fn f<T>()where <T as A>::Out:Copy{}fn main(){}",
        true,
    ),
    (
        "type-copy-called",
        "trait A{type Out;}struct N;impl<T>A for T{type Out=N;}fn f<T>()where <T as A>::Out:Copy{}fn main(){f::<u8>();}",
        false,
    ),
    (
        "erased-bound-lifetime-call",
        "trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=u8;}fn f<'a>()where <u8 as A<'a>>::Out:Copy{}fn main(){f::<'static>();}",
        true,
    ),
];
#[test]
fn source_signatures_wait_for_their_parameter_environment() -> Result<(), Box<dyn std::error::Error>>
{
    let mut failures = Vec::new();
    for &(name, source, accepted) in CASES {
        rustc::qualify(&format!("signature-environment-{name}"), source, accepted)?;
        let result = semantic::analyze(source).and_then(|(mut report, mut facts)| {
            let entry =
                rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
            rustgm::lower::instances::build(&report, &mut facts, entry)
                .map(|_| ())
                .map_err(|e| e.to_string())
        });
        if result.is_ok() != accepted {
            failures.push(format!("{name}: {:?}", result.err()));
        } else if name == "lifetime-copy-called"
            && result
                .as_ref()
                .err()
                .is_none_or(|error| !error.contains("must implement Copy"))
        {
            failures.push(format!(
                "{name}: rejected before its Copy requirement: {:?}",
                result.err()
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn caller_bounds_select_executable_projected_signatures() -> Result<(), Box<dyn std::error::Error>>
{
    native::check_cases(
        "signature-environments",
        &[
            (
                "copy-output",
                "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}fn f<T:Copy>()-><T as A>::Out{73}fn main(){check(f::<u8>()==73)}",
                false,
            ),
            (
                "user-trait-input",
                "trait B{}impl B for u8{}trait A{type Out;}impl<T:B>A for T{type Out=u8;}fn f<T:B>(x:<T as A>::Out)->u8{x+1}fn main(){check(f::<u8>(73)==74)}",
                false,
            ),
            (
                "method-inheritance",
                "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}struct S<T>(T);impl<T:Copy>S<T>{fn f(x:<T as A>::Out)->u8{x+1}}fn main(){check(S::<u16>::f(73)==74)}",
                false,
            ),
            (
                "normalized-bound-subject",
                "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B{}impl B for u8{}fn f<T>()-><T as A>::Out where <T as A>::Out:B,T:Copy{73}fn main(){check(f::<u16>()==73)}",
                false,
            ),
            (
                "selected-nominal-default",
                "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}struct S<T:Copy,U=<T as A>::Out>(T,U);fn main(){let s:S<u8>=S(1u8,73u8);check(s.1==73)}",
                false,
            ),
            (
                "lifetime-only-environment",
                "trait B<'a>{}impl<'a>B<'a>for u8{}trait A<'a>{type Out;}impl<'a>A<'a>for u8 where u8:B<'a>{type Out=u8;}fn f<'a>()-><u8 as A<'a>>::Out where u8:B<'a>{73}fn main(){check(f::<'static>()==73)}",
                false,
            ),
        ],
    )
}
