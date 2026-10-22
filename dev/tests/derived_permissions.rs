//! Derived result permissions retain parent applications and exact caller choices.
use crate::native_references as native;
use crate::qualify as rustc;
use crate::semantic;

const CASES: &[(&str, &str, bool)] = &[
    (
        "symbolic-permission",
        "trait C{}trait A{type Out:C;}fn need<T:C>(){}fn f<T:A>(){need::<<T as A>::Out>();}fn main(){}",
        true,
    ),
    (
        "missing-permission",
        "trait C{}trait A{type Out;}fn need<T:C>(){}fn f<T:A>(){need::<<T as A>::Out>();}fn main(){}",
        false,
    ),
    (
        "transitive-supertrait",
        "trait C{}trait D:C{}trait A{type Out:D;}fn need<T:C>(){}fn f<T:A>(){need::<<T as A>::Out>();}fn main(){}",
        true,
    ),
    (
        "recursive-bound-demand",
        "trait A{type Out:A;}fn need<T:A>(){}fn f<T:A>(){need::<<<T as A>::Out as A>::Out>();}fn main(){}",
        true,
    ),
    (
        "same-output-inputs-distinct",
        "trait C<T:?Sized>{}trait A{type Out:C<Self>;}fn need<T:C<u8>>(){}fn f<T:A>(){need::<<T as A>::Out>();}fn main(){}",
        false,
    ),
    (
        "derived-self",
        "trait C<T:?Sized>{}trait A{type Out:C<Self>;}fn need<S:?Sized,T:C<S>>(){}fn f<T:A>(){need::<T,<T as A>::Out>();}fn main(){}",
        true,
    ),
    (
        "derived-type-argument",
        "trait C<T>{}trait A<T>{type Out:C<T>;}fn need<S,T:C<S>>(){}fn f<S,T:A<S>>(){need::<S,<T as A<S>>::Out>();}fn main(){}",
        true,
    ),
    (
        "wrong-derived-type-argument",
        "trait C<T>{}trait A<T>{type Out:C<T>;}fn need<T:C<u16>>(){}fn f<T:A<u8>>(){need::<<T as A<u8>>::Out>();}fn main(){}",
        false,
    ),
    (
        "derived-const",
        "trait C<const N:usize>{}trait A<const N:usize>{type Out:C<N>;}fn need<T:C<3>>(){}fn f<T:A<3>>(){need::<<T as A<3>>::Out>();}fn main(){}",
        true,
    ),
    (
        "wrong-derived-const",
        "trait C<const N:usize>{}trait A<const N:usize>{type Out:C<N>;}fn need<T:C<4>>(){}fn f<T:A<3>>(){need::<<T as A<3>>::Out>();}fn main(){}",
        false,
    ),
    (
        "derived-lifetime",
        "trait C<'a>{}trait A<'a>{type Out:C<'a>;}fn need<'a,T:C<'a>>(){}fn f<'a,T:A<'a>>(){need::<'a,<T as A<'a>>::Out>();}fn main(){}",
        true,
    ),
    (
        "wrong-derived-lifetime",
        "trait C<'a>{}trait A<'a>{type Out:C<'a>;}fn need<'a,T:C<'a>>(){}fn f<'a:'b,'b,T:A<'a>>(){need::<'b,<T as A<'a>>::Out>();}fn main(){}",
        false,
    ),
    (
        "equal-derived-lifetime",
        "trait C<'a>{}trait A<'a>{type Out:C<'a>;}fn need<'a,T:C<'a>>(){}fn f<'a:'b,'b:'a,T:A<'a>>(){need::<'b,<T as A<'a>>::Out>();}fn main(){}",
        true,
    ),
    (
        "missing-parent-premise",
        "trait C{}trait A{type Out:C;}fn need<T:C>(){}fn f<T>(){need::<<T as A>::Out>();}fn main(){}",
        false,
    ),
    (
        "wrong-parent-arguments",
        "trait C{}trait A<T>{type Out:C;}fn need<T:C>(){}fn f<T:A<u8>>(){need::<<T as A<u16>>::Out>();}fn main(){}",
        false,
    ),
    (
        "recursive-four",
        "trait A{type Out:A;}fn need<T:A>(){}fn f<T:A>(){need::<<<<<T as A>::Out as A>::Out as A>::Out as A>::Out>();}fn main(){}",
        true,
    ),
    (
        "indirect-result-cycle",
        "trait A{type Out:B;}trait B{type Out:A;}fn need<T:A>(){}fn f<T:A>(){need::<<<T as A>::Out as B>::Out>();}fn main(){}",
        true,
    ),
    (
        "unrelated-result-bound",
        "trait C{}trait D{}trait A{type Out:C;}fn need<T:D>(){}fn f<T:A>(){need::<<T as A>::Out>();}fn main(){}",
        false,
    ),
    (
        "sibling-has-no-bound",
        "trait C{}trait A{type X:C;type Y;}fn need<T:C>(){}fn f<T:A>(){need::<<T as A>::Y>();}fn main(){}",
        false,
    ),
    (
        "derived-supertrait-argument",
        "trait C<T>{}trait D<T>:C<T>{}trait A<T>{type Out:D<T>;}fn need<S,T:C<S>>(){}fn f<S,T:A<S>>(){need::<S,<T as A<S>>::Out>();}fn main(){}",
        true,
    ),
    (
        "wrong-supertrait-argument",
        "trait C<T>{}trait D<T>:C<T>{}trait A<T>{type Out:D<T>;}fn need<T:C<u16>>(){}fn f<T:A<u8>>(){need::<<T as A<u8>>::Out>();}fn main(){}",
        false,
    ),
    (
        "supertrait-lifetime-capture",
        "trait C<'a,T>{}trait D<'a,T>:C<'a,T>{}trait A<'a,T>{type Out:D<'a,T>;}fn need<'a,S,T:C<'a,S>>(){}fn f<'a,S,T:A<'a,S>>(){need::<'a,S,<T as A<'a,S>>::Out>();}fn main(){}",
        true,
    ),
    (
        "wrong-supertrait-lifetime-capture",
        "trait C<'a,T>{}trait D<'a,T>:C<'a,T>{}trait A<'a,T>{type Out:D<'a,T>;}fn need<'a,S,T:C<'a,S>>(){}fn f<'a:'b,'b,S,T:A<'a,S>>(){need::<'b,S,<T as A<'a,S>>::Out>();}fn main(){}",
        false,
    ),
    (
        "inferred-argument",
        "trait C<T>{}trait A{type Out:C<u8>;}fn need<S,T:C<S>>(){}fn f<T:A>(){need::<_,<T as A>::Out>();}fn main(){}",
        true,
    ),
    (
        "ambiguous-result-arguments",
        "trait C<T>{}trait A{type Out:C<u8>+C<u16>;}fn need<S,T:C<S>>(){}fn f<T:A>(){need::<_,<T as A>::Out>();}fn main(){}",
        false,
    ),
    (
        "constrained-result-arguments",
        "trait C<T>{}trait A{type Out:C<u8>+C<u16>;}fn need<S,T:C<S>>()->S{loop{}}fn f<T:A>(){let _:u8=need::<_,<T as A>::Out>();}fn main(){}",
        true,
    ),
    (
        "environment-bound-preference",
        "trait C<T>{}trait A{type Out:C<u16>;}fn need<S,T:C<S>>(){}fn f<T:A>()where <T as A>::Out:C<u8>{need::<_,<T as A>::Out>();}fn main(){}",
        true,
    ),
    (
        "explicit-environment-and-result",
        "trait C<T>{}trait A{type Out:C<u16>;}fn need<S,T:C<S>>(){}fn f<T:A>()where <T as A>::Out:C<u8>{need::<u16,<T as A>::Out>();need::<u8,<T as A>::Out>();}fn main(){}",
        true,
    ),
    (
        "duplicate-environment-and-result",
        "trait C<T>{}trait A{type Out:C<u8>;}fn need<S,T:C<S>>(){}fn f<T:A>()where <T as A>::Out:C<u8>{need::<_,<T as A>::Out>();}fn main(){}",
        true,
    ),
    (
        "unsized-associated-result",
        "trait C{}trait A{type Out:?Sized+C;}fn need<T:?Sized+C>(){}fn f<T:A>(){need::<<T as A>::Out>();}fn main(){}",
        true,
    ),
    (
        "derived-implementation-premise",
        "trait C{}trait A{type Out:C;}trait R{}impl<T:A>R for T where <T as A>::Out:C{}fn need<T:R>(){}fn f<T:A>(){need::<T>();}fn main(){}",
        true,
    ),
    (
        "missing-implementation-premise",
        "trait C{}trait A{type Out;}trait R{}impl<T:A>R for T where <T as A>::Out:C{}fn need<T:R>(){}fn f<T:A>(){need::<T>();}fn main(){}",
        false,
    ),
    (
        "different-parent-permissions",
        "trait C{}trait D{}trait A{type Out:C;}fn need<T:D>(){}fn f<T:A,U:A>()where <T as A>::Out:D{need::<<U as A>::Out>();}fn main(){}",
        false,
    ),
    (
        "function-pointer-result",
        "trait A{type Out;}fn need<T>(){}fn f<T:A>()->fn(){need::<<T as A>::Out>}fn main(){}",
        true,
    ),
    (
        "missing-result-static",
        "trait C{}trait A{type Out:C;}fn need<T:'static>(){}fn f<T:A>(){need::<<T as A>::Out>();}fn main(){}",
        false,
    ),
    (
        "lifetime-argument-is-not-result-bound",
        "trait C<'a>{}trait A<'a>{type Out:C<'a>;}fn need<T:'static>(){}fn f<'a,T:A<'a>>(){need::<<T as A<'a>>::Out>();}fn main(){}",
        false,
    ),
];
#[test]
fn result_permissions_require_complete_parent_and_argument_evidence()
-> Result<(), Box<dyn std::error::Error>> {
    for &(name, source, accepted) in CASES {
        rustc::qualify(&format!("derived-result-{name}"), source, accepted)?;
        let result = analyze(source);
        assert_eq!(result.is_ok(), accepted, "{name}: {result:?}");
    }
    Ok(())
}

#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn derived_result_permissions_select_the_correct_executable_body()
-> Result<(), Box<dyn std::error::Error>> {
    native::check_cases(
        "derived-result-permissions",
        &[
            (
                "direct",
                "trait C{fn n()->u8;}impl C for u16{fn n()->u8{73}}trait A{type Out:C;}impl A for u8{type Out=u16;}fn f<T:A>()->u8{<<T as A>::Out as C>::n()}fn main(){check(f::<u8>()==73);}",
                false,
            ),
            (
                "supertrait",
                "trait C{fn n()->u8;}trait D:C{}impl C for u16{fn n()->u8{73}}impl D for u16{}trait A{type Out:D;}impl A for u8{type Out=u16;}fn f<T:A>()->u8{<<T as A>::Out as C>::n()}fn main(){check(f::<u8>()==73);}",
                false,
            ),
            (
                "recursive",
                "trait C{fn n()->u8;}trait A:C{type Out:A;}impl C for u8{fn n()->u8{1}}impl C for u16{fn n()->u8{2}}impl C for u32{fn n()->u8{73}}impl A for u8{type Out=u16;}impl A for u16{type Out=u32;}impl A for u32{type Out=u32;}fn f<T:A>()->u8{<<<T as A>::Out as A>::Out as C>::n()}fn main(){check(f::<u8>()==73);}",
                false,
            ),
            (
                "implementation-premise",
                "trait C{}impl C for u16{}trait A{type Out:C;}impl A for u8{type Out=u16;}trait R{fn n()->u8;}impl<T:A>R for T where <T as A>::Out:C{fn n()->u8{73}}fn f<T:A>()->u8{<T as R>::n()}fn main(){check(f::<u8>()==73);}",
                false,
            ),
            (
                "mixed-arguments",
                "trait C<'a,T,const N:usize>{fn n()->u8;}impl<'a>C<'a,u8,3>for u16{fn n()->u8{73}}trait A<'a,T,const N:usize>{type Out:C<'a,T,N>;}impl<'a>A<'a,u8,3>for u32{type Out=u16;}fn f<'a,U:A<'a,u8,3>>()->u8{<<U as A<'a,u8,3>>::Out as C<'a,u8,3>>::n()}fn main(){check(f::<'static,u32>()==73);}",
                false,
            ),
            (
                "caller-bound-preference",
                "trait C<T>{fn n()->u8;}trait A{type Out:C<u16>;}impl C<u8>for u32{fn n()->u8{73}}impl C<u16>for u32{fn n()->u8{99}}impl A for u64{type Out=u32;}fn call<S,T:C<S>>()->u8{<T as C<S>>::n()}fn f<T:A>()->u8 where <T as A>::Out:C<u8>{call::<_,<T as A>::Out>()}fn main(){check(f::<u64>()==73);}",
                false,
            ),
        ],
    )
}

fn analyze(source: &str) -> Result<(), String> {
    let (mut report, mut facts) = semantic::analyze(source)?;
    let entry = rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
    rustgm::lower::instances::build(&report, &mut facts, entry)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[test]
fn wide_repeated_permissions_share_rules_without_granting_an_unrelated_trait()
-> Result<(), Box<dyn std::error::Error>> {
    use std::fmt::Write;
    for (width, scopes, accepted) in [
        (1024, false, true),
        (1024, true, true),
        (64, false, false),
        (64, true, false),
    ] {
        let mut source = String::new();
        for index in 0..width {
            write!(source, "trait C{index}{{}}")?;
        }
        source.push_str("trait A{type Out:");
        for index in 0..width {
            if index != 0 {
                source.push('+');
            }
            write!(source, "C{index}")?;
        }
        write!(
            source,
            ";}}fn need<T:C{}>(){{}}trait Missing{{}}fn missing<T:Missing>(){{}}fn f<T:A>(){{",
            width - 1
        )?;
        for index in 0..width {
            if scopes {
                write!(source, "}}fn f{index}<T:A>(){{")?;
            }
            source.push_str("need::<<T as A>::Out>();");
        }
        if !accepted {
            source.push_str("missing::<<T as A>::Out>();");
        }
        source.push_str("}fn main(){}");
        rustc::qualify(
            &format!("wide-result-permissions-{width}-{scopes}"),
            &source,
            accepted,
        )?;
        let result = analyze(&source);
        assert_eq!(result.is_ok(), accepted, "width{width}: {result:?}");
    }
    Ok(())
}
