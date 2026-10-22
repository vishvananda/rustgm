//! Coherence excludes a Sized blanket only with an independently justified tail.
use crate::native_references as native;
use crate::qualify as rustc;
use crate::semantic;

const CASES: &[(&str, bool, &str)] = &[
    (
        "closed-unsized",
        true,
        "trait A{type Out:?Sized;}impl A for u8{type Out=[u8];}struct S<T:A>{tail:T::Out}trait B{}impl<T>B for T{}impl B for S<u8>{}",
    ),
    (
        "closed-sized",
        false,
        "trait A{type Out:?Sized;}impl A for u8{type Out=u8;}struct S<T:A>{tail:T::Out}trait B{}impl<T>B for T{}impl B for S<u8>{}",
    ),
    (
        "open-projection",
        false,
        "trait A{type Out:?Sized;}struct S<T:A>{tail:T::Out}trait B{}impl<T>B for T{}impl<T:A>B for S<T>{}",
    ),
    (
        "generic-fixed-projection",
        false,
        "trait A{type Out:?Sized;}impl<T>A for T{type Out=[u8];}struct S<T:A>{tail:T::Out}trait B{}impl<T>B for T{}impl<T>B for S<T>{}",
    ),
    (
        "generic-concrete-tail",
        true,
        "struct S<T>{prefix:T,tail:[u8]}trait B{}impl<T>B for T{}impl<T>B for S<T>{}",
    ),
    (
        "direct-control",
        true,
        "struct S<T:?Sized>{tail:T}trait B{}impl<T>B for T{}impl B for S<[u8]>{}",
    ),
    (
        "tuple-control",
        true,
        "trait B{}impl<T>B for T{}impl B for (u8,[u8]){}",
    ),
    (
        "nested-projection",
        true,
        "trait A{type Out:?Sized;}impl A for u8{type Out=[u8];}struct S<T:A>{tail:T::Out}struct W<T:?Sized>{tail:T}trait B{}impl<T>B for T{}impl B for W<S<u8>>{}",
    ),
    (
        "static-projection",
        true,
        "trait A{type Out:?Sized;}impl<'a>A for &'a u8{type Out=[u8];}struct S<T:A>{tail:T::Out}trait B{}impl<T>B for T{}impl B for S<&'static u8>{}",
    ),
    (
        "generic-lifetime-projection",
        true,
        "trait A{type Out:?Sized;}impl<'a>A for &'a u8{type Out=[u8];}struct S<T:A>{tail:T::Out}trait B{}impl<T>B for T{}impl<'a>B for S<&'a u8>{}",
    ),
    (
        "copy-blanket",
        true,
        "trait A{type Out:?Sized;}impl A for u8{type Out=[u8];}struct S<T:A>{tail:T::Out}trait B{}impl<T:Copy>B for T{}impl B for S<u8>{}",
    ),
    (
        "missing-static-premise",
        false,
        "trait A{type Out:?Sized;}impl<T:'static>A for T{type Out=[u8];}struct S<T:A>{tail:T::Out}trait B{}impl<T>B for T{}impl<'a>B for S<&'a u8>{}",
    ),
    (
        "static-premise",
        true,
        "trait A{type Out:?Sized;}impl<T:'static>A for T{type Out=[u8];}struct S<T:A>{tail:T::Out}trait B{}impl<T>B for T{}impl<'a>B for S<&'a u8>where 'a:'static{}",
    ),
    (
        "ordered-lifetimes",
        true,
        "trait A<'b>{type Out:?Sized;}impl<'a:'b,'b>A<'b> for &'a u8{type Out=[u8];}struct S<'b,T:A<'b>>{tail:T::Out}trait B{}impl<T>B for T{}impl<'a:'b,'b>B for S<'b,&'a u8>{}",
    ),
    (
        "unordered-lifetimes",
        false,
        "trait A<'b>{type Out:?Sized;}impl<'a:'b,'b>A<'b> for &'a u8{type Out=[u8];}struct S<'b,T:A<'b>>{tail:T::Out}trait B{}impl<T>B for T{}impl<'a,'b>B for S<'b,&'a u8>{}",
    ),
];

#[test]
fn storage_disjointness_retains_header_lifetime_premises() -> Result<(), Box<dyn std::error::Error>>
{
    for &(name, expected, source) in CASES {
        rustc::qualify(&format!("tail-overlap-{name}"), source, expected)?;
        let result = semantic::analyze(source);
        assert_eq!(result.is_ok(), expected, "{name}: {:?}", result.err());
    }
    Ok(())
}

#[test]
#[ignore = "requires the qualified CPPGM native backend"]
fn selected_unsized_implementations_execute_beside_sized_blankets()
-> Result<(), Box<dyn std::error::Error>> {
    native::check_cases(
        "pa22-native/tail-overlap",
        &[
            (
                "closed",
                "trait A{type Out:?Sized;}impl A for u8{type Out=[u8];}struct S<T:A>{tail:T::Out}trait B{fn n()->u8;}impl<T>B for T{fn n()->u8{1}}impl B for S<u8>{fn n()->u8{2}}fn main(){check(<S<u8> as B>::n()==2);check(<u8 as B>::n()==1);}",
                false,
            ),
            (
                "generic",
                "struct S<T>{prefix:T,tail:[u8]}trait B{fn n()->u8;}impl<T>B for T{fn n()->u8{1}}impl<T>B for S<T>{fn n()->u8{3}}fn main(){check(<S<u8> as B>::n()==3);check(<S<u16> as B>::n()==3);check(<u8 as B>::n()==1);}",
                false,
            ),
            (
                "lifetimes",
                "trait A{type Out:?Sized;}impl<'a>A for &'a u8{type Out=[u8];}struct S<T:A>{tail:T::Out}trait B{fn n()->u8;}impl<T>B for T{fn n()->u8{1}}impl<'a>B for S<&'a u8>{fn n()->u8{4}}fn f<'a>(_:&'a u8)->u8{<S<&'a u8> as B>::n()}fn main(){check(f(&7)==4);check(<u8 as B>::n()==1);}",
                false,
            ),
        ],
    )
}
