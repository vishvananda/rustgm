//! Associated result contracts are checked on declarations and every implementing type.
use crate::native_references as native;
use crate::qualify as rustc;
use crate::semantic;

const CASES: &[(&str, &str, bool)] = &[
    (
        "concrete",
        "trait C{}impl C for u16{}trait A{type Out:C;}impl A for u8{type Out=u16;}fn main(){}",
        true,
    ),
    (
        "invalid-unused-definition",
        "trait C{}impl C for u16{}trait A{type Out:C;}impl A for u8{type Out=u32;}fn main(){}",
        false,
    ),
    (
        "substitute-self",
        "trait C<T:?Sized>{}impl C<u8>for u16{}trait A{type Out:C<Self>;}impl A for u8{type Out=u16;}fn main(){}",
        true,
    ),
    (
        "wrong-self",
        "trait C<T:?Sized>{}impl C<u16>for u16{}trait A{type Out:C<Self>;}impl A for u8{type Out=u16;}fn main(){}",
        false,
    ),
    (
        "substitute-argument",
        "trait C<T>{}impl C<u8>for u32{}trait A<T>{type Out:C<T>;}impl A<u8>for u16{type Out=u32;}fn main(){}",
        true,
    ),
    (
        "wrong-argument",
        "trait C<T>{}impl C<u16>for u32{}trait A<T>{type Out:C<T>;}impl A<u8>for u16{type Out=u32;}fn main(){}",
        false,
    ),
    (
        "forward-bound",
        "trait A{type Out:C;}trait C{}impl C for u16{}impl A for u8{type Out=u16;}fn main(){}",
        true,
    ),
    (
        "recursive-bound-unused",
        "trait A{type Out:A;}fn f<T:A>(){}fn main(){}",
        true,
    ),
    (
        "recursive-bound-concrete",
        "trait A{type Out:A;}impl A for u8{type Out=u8;}fn need<T:A>(){}fn main(){need::<<<u8 as A>::Out as A>::Out>();}",
        true,
    ),
    (
        "nested-equality-mismatch",
        "trait C{type Out;}impl C for u16{type Out=u32;}trait A{type Out:C<Out=u8>;}impl A for u8{type Out=u16;}fn main(){}",
        false,
    ),
    (
        "unsized-self-argument",
        "trait C<T>{}trait A{type Out:C<Self>;}fn main(){}",
        false,
    ),
    (
        "sized-self-argument",
        "trait C<T>{}trait A:Sized{type Out:C<Self>;}fn main(){}",
        true,
    ),
    (
        "unsized-parameter-argument",
        "trait C<T>{}trait A<T:?Sized>{type Out:C<T>;}fn main(){}",
        false,
    ),
    (
        "missing-trait-argument-bound",
        "trait D{}trait C<T:D>{}trait A<T>{type Out:C<T>;}fn main(){}",
        false,
    ),
    (
        "valid-trait-argument-bound",
        "trait D{}trait C<T:D>{}trait A<T:D>{type Out:C<T>;}fn main(){}",
        true,
    ),
    (
        "missing-lifetime-argument-bound",
        "trait C<'a,T:'a>{}trait A<'a,T>{type Out:C<'a,T>;}fn main(){}",
        false,
    ),
    (
        "valid-lifetime-argument-bound",
        "trait C<'a,T:'a>{}trait A<'a,T:'a>{type Out:C<'a,T>;}fn main(){}",
        true,
    ),
    (
        "mixed-substitution",
        "trait C<'a,T,const N:usize>{}impl<'a>C<'a,u8,3>for u16{}trait A<'a,T,const N:usize>{type Out:C<'a,T,N>;}impl<'a>A<'a,u8,3>for u32{type Out=u16;}fn main(){}",
        true,
    ),
    (
        "wrong-const-substitution",
        "trait C<'a,T,const N:usize>{}impl<'a>C<'a,u8,4>for u16{}trait A<'a,T,const N:usize>{type Out:C<'a,T,N>;}impl<'a>A<'a,u8,3>for u32{type Out=u16;}fn main(){}",
        false,
    ),
    (
        "wrong-lifetime-substitution",
        "trait C<'a>{}impl<'a>C<'a>for &'a u8{}trait A<'a>{type Out:C<'a>;}impl<'a,'b>A<'a>for &'b u8{type Out=&'b u8;}fn main(){}",
        false,
    ),
    (
        "valid-lifetime-substitution",
        "trait C<'a>{}impl<'a>C<'a>for &'a u8{}trait A<'a>{type Out:C<'a>;}impl<'a:'b,'b:'a>A<'a>for &'b u8{type Out=&'b u8;}fn main(){}",
        true,
    ),
    (
        "nested-equality-concrete",
        "trait C{type Out;}impl C for u16{type Out=u8;}trait A{type Out:C<Out=u8>;}impl A for u8{type Out=u16;}fn main(){}",
        true,
    ),
    (
        "generic-self-and-argument",
        "struct W<T>(T);trait C<S,T>{}trait A<T>:Sized{type Out:C<Self,T>;}impl<U>C<W<U>,U>for U{}impl<T>A<T>for W<T>{type Out=T;}fn main(){}",
        true,
    ),
];
#[test]
fn result_bounds_require_valid_declarations_and_implementation_proofs()
-> Result<(), Box<dyn std::error::Error>> {
    for &(name, source, accepted) in CASES {
        rustc::qualify(&format!("associated-result-bound-{name}"), source, accepted)?;
        let result = semantic::analyze(source).and_then(|(mut report, mut facts)| {
            let entry =
                rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
            rustgm::lower::instances::build(&report, &mut facts, entry)
                .map(|_| ())
                .map_err(|e| e.to_string())
        });
        assert_eq!(result.is_ok(), accepted, "{name}: {result:?}");
    }
    Ok(())
}

#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn associated_result_contracts_reach_native_selection() -> Result<(), Box<dyn std::error::Error>> {
    native::check_cases(
        "associated-result-bounds",
        &[
            (
                "concrete",
                "trait C{fn n()->u8;}impl C for u16{fn n()->u8{73}}trait A{type Out:C;}impl A for u8{type Out=u16;}fn main(){check(<<u8 as A>::Out as C>::n()==73);}",
                false,
            ),
            (
                "type-argument",
                "trait C<T>{fn n()->u8;}impl C<u8>for u32{fn n()->u8{73}}trait A<T>{type Out:C<T>;}impl A<u8>for u16{type Out=u32;}fn main(){check(<<u16 as A<u8>>::Out as C<u8>>::n()==73);}",
                false,
            ),
            (
                "nested-equality",
                "trait C{type Item;fn value()->u8;}impl C for u16{type Item=u8;fn value()->u8{73}}trait A{type Out:C<Item=u8>;}impl A for u32{type Out=u16;}fn main(){let value:<<u32 as A>::Out as C>::Item=<<u32 as A>::Out as C>::value();check(value==73u8);}",
                false,
            ),
        ],
    )
}
