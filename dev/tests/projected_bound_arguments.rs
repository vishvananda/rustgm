//! Complete trait arguments retain projected bound identities and caller requirements.
use crate::native_references as native;
use crate::qualify as rustc;
use crate::semantic;
const CASES: &[(&str, &str, bool)] = &[
    (
        "inline-argument",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<U>{}fn f<T:Copy,U:B<<T as A>::Out>>(){}fn main(){}",
        true,
    ),
    (
        "argument-before",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<U>{}fn f<T,U>()where U:B<<T as A>::Out>,T:Copy{}fn main(){}",
        true,
    ),
    (
        "argument-after",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<U>{}fn f<T,U>()where T:Copy,U:B<<T as A>::Out>{}fn main(){}",
        true,
    ),
    (
        "argument-missing-selection",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<U>{}fn f<T,U>()where U:B<<T as A>::Out>{}fn main(){}",
        false,
    ),
    (
        "composite-argument",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<U>{}fn f<T,U>()where U:B<(<T as A>::Out,u16)>,T:Copy{}fn main(){}",
        true,
    ),
    (
        "reference-argument",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<U>{}fn f<'a,T,U>()where U:B<&'a <T as A>::Out>,T:Copy{}fn main(){}",
        true,
    ),
    (
        "subject-and-argument",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<U>{}fn f<T>()where <T as A>::Out:B<<T as A>::Out>,T:Copy{}fn main(){}",
        true,
    ),
    (
        "closed-result-unused",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<U>{}fn f<T>()where u8:B<<T as A>::Out>,T:Copy{}fn main(){}",
        true,
    ),
    (
        "closed-result-called",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<U>{}fn f<T>()where u8:B<<T as A>::Out>,T:Copy{}fn main(){f::<u8>();}",
        false,
    ),
    (
        "closed-written-result",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<U>{}fn f<T>()where u8:B<<u8 as A>::Out>,T:Copy{}fn main(){}",
        false,
    ),
    (
        "normalized-argument-called",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<U>{}impl B<u8>for u16{}fn f<T,U>()where U:B<<T as A>::Out>,T:Copy{}fn main(){f::<u8,u16>();}",
        true,
    ),
    (
        "signature-shares-environment",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<U>{}fn f<T,U>(x:<T as A>::Out)-><T as A>::Out where U:B<<T as A>::Out>,T:Copy{x}fn main(){}",
        true,
    ),
    (
        "method-inherited-copy",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<U>{}struct S<T>(T);impl<T:Copy>S<T>{fn f<U:B<<T as A>::Out>>() {}}fn main(){}",
        true,
    ),
    (
        "lifetime-only-argument",
        "trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=u8;}trait B<U>{}fn f<'a>()where u8:B<<u8 as A<'a>>::Out>{}fn main(){}",
        true,
    ),
    (
        "early-lifetime-call",
        "trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=u8;}trait B<U>{}impl B<u8>for u8{}fn f<'a>()where u8:B<<u8 as A<'a>>::Out>{}fn main(){f::<'static>();}",
        true,
    ),
    (
        "nested-bound-projection",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<U>{}fn f<T,U>()where U:B<<<T as A>::Out as A>::Out>,T:Copy{}fn main(){}",
        true,
    ),
    (
        "argument-selection-dependency",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<U>{}trait C{type Out;}impl<T:B<u8>>C for T{type Out=u16;}fn f<T,U>(x:<U as C>::Out)where U:B<<T as A>::Out>,T:Copy{}fn main(){}",
        true,
    ),
    (
        "mixed-arguments",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<'a,U,const N:usize>{}fn f<'a,T:Copy,U:B<'a,<T as A>::Out,3>>(){}fn main(){}",
        true,
    ),
    (
        "array-argument",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<U>{}fn f<T:Copy,U:B<[<T as A>::Out;0]>>(){}fn main(){}",
        true,
    ),
    (
        "array-argument-requires-selection",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<U>{}fn f<T,U:B<[<T as A>::Out;0]>>(){}fn main(){}",
        false,
    ),
    (
        "nominal-argument",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<U>{}struct W<T>(T);fn f<T:Copy,U:B<W<<T as A>::Out>>>(){}fn main(){}",
        true,
    ),
    (
        "multiple-arguments",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<X,Y>{}fn f<T:Copy,U:B<<T as A>::Out,<T as A>::Out>>(){}fn main(){}",
        true,
    ),
    (
        "argument-generic-lifetime",
        "trait A<'a>{type Out;}impl<'a,T:Copy>A<'a>for T{type Out=u8;}trait B<U>{}fn f<'a,T:Copy,U:B<<T as A<'a>>::Out>>(){}fn main(){}",
        true,
    ),
    (
        "early-lifetime-missing-trait",
        "trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=u8;}trait B<U>{}fn f<'a>()where u8:B<<u8 as A<'a>>::Out>{}fn main(){f::<'static>();}",
        false,
    ),
    (
        "closed-argument-unused",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<U>{}fn f<U:B<<u8 as A>::Out>>(){}fn main(){}",
        true,
    ),
    (
        "elided-bound-reference",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<U>{}fn f<U:B<&u8>>(){}fn main(){}",
        false,
    ),
    (
        "placeholder-bound-reference",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<U>{}fn f<U:B<&'_ u8>>(){}fn main(){}",
        false,
    ),
    (
        "normalized-body-call",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<U>{fn value()->U;}fn f<T:Copy,U:B<<T as A>::Out>>()->u8{<U as B<u8>>::value()}fn main(){}",
        true,
    ),
    (
        "normalized-supertrait-body",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<U>{fn value()->U;}trait D<U>:B<U>{}fn f<T:Copy,U:D<<T as A>::Out>>()->u8{<U as B<u8>>::value()}fn main(){}",
        true,
    ),
    (
        "unrelated-owner-bound",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<U>{}fn f<T:Copy,U:B<<T as A>::Out>>(){}fn g<T,U:B<<T as A>::Out>>(){}fn main(){}",
        false,
    ),
    (
        "alias-wrapper-argument",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<U>{}type Id<T>=T;fn f<T:Copy,U:B<Id<<T as A>::Out>>>(){}fn main(){}",
        true,
    ),
    (
        "defaulted-wrapper-argument",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<U>{}struct W<T,U=T>(T,U);fn f<T:Copy,U:B<W<<T as A>::Out>>>(){}fn main(){}",
        true,
    ),
    (
        "nominal-bound-subject",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<U>{}struct W<T>(T);fn f<T:Copy>()where W<<T as A>::Out>:Copy{}fn main(){}",
        true,
    ),
    (
        "nominal-bound-subject-missing",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<U>{}struct W<T>(T);fn f<T>()where W<<T as A>::Out>:Copy{}fn main(){}",
        false,
    ),
    (
        "callable-elided-copy",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}fn f<T>()where (fn(&u8)-><T as A>::Out):Copy,T:Copy{}fn main(){f::<u8>();}",
        true,
    ),
    (
        "callable-elided-missing",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}fn f<T>()where (fn(&u8)-><T as A>::Out):Copy{}fn main(){}",
        false,
    ),
    (
        "callable-elided-argument",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<U>{}fn f<T,U>()where U:B<fn(&u8)-><T as A>::Out>,T:Copy{}fn main(){}",
        true,
    ),
];
#[test]
fn trait_argument_projections_use_complete_bound_environments()
-> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, source, accepted) in CASES {
        rustc::qualify(
            &format!("projected-bound-argument-{name}"),
            source,
            accepted,
        )?;
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
            let message = match name {
                "closed-result-called" | "early-lifetime-missing-trait" => {
                    "trait obligation requires an unambiguous matching assumption or implementation"
                }
                "closed-written-result" => "closed trait bounds require an implementation proof",
                "elided-bound-reference" | "placeholder-bound-reference" => {
                    "bound references require explicit named lifetimes"
                }
                _ => "associated type requires an unambiguous implementation",
            };
            if result
                .as_ref()
                .err()
                .is_none_or(|error| !error.contains(message))
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
fn normalized_trait_arguments_execute() -> Result<(), Box<dyn std::error::Error>> {
    native::check_cases(
        "projected-bound-arguments",
        &[
            (
                "generic-trait-call",
                "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<U>{fn value()->U;}impl B<u8>for u16{fn value()->u8{73}}fn f<T:Copy,U:B<<T as A>::Out>>()->u8{<U as B<u8>>::value()}fn main(){check(f::<u8,u16>()==73)}",
                false,
            ),
            (
                "method-trait-call",
                "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<U>{fn value()->U;}impl B<u8>for u16{fn value()->u8{73}}struct S<T>(T);impl<T:Copy>S<T>{fn f<U:B<<T as A>::Out>>()->u8{<U as B<u8>>::value()}}fn main(){check(S::<u8>::f::<u16>()==73)}",
                false,
            ),
            (
                "normalized-supertrait-call",
                "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<U>{fn value()->U;}trait D<U>:B<U>{}impl B<u8>for u16{fn value()->u8{73}}impl D<u8>for u16{}fn f<T:Copy,U:D<<T as A>::Out>>()->u8{<U as B<u8>>::value()}fn main(){check(f::<u8,u16>()==73)}",
                false,
            ),
            (
                "mixed-trait-arguments",
                "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<'a,U,const N:usize>{fn value()->U;}impl<'a>B<'a,u8,3>for u16{fn value()->u8{73}}fn f<'a,T:Copy,U:B<'a,<T as A>::Out,3>>()->u8{<U as B<'a,u8,3>>::value()}fn main(){check(f::<'static,u8,u16>()==73)}",
                false,
            ),
        ],
    )
}
