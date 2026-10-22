//! Inherited method contracts retain explicit generic bounds and concrete result evidence.
use crate::native_references as native;
use crate::qualify as rustc;
use crate::semantic;

const CASES: &[(&str, &str, bool)] = &[
    (
        "closed-result",
        "trait A{type Out;}trait C:A{fn get()-><Self as A>::Out;}impl A for u8{type Out=u16;}impl C for u8{fn get()->u16{73}}fn main(){let _=<u8 as C>::get();}\n",
        true,
    ),
    (
        "wrong-result",
        "trait A{type Out;}trait C:A{fn get()-><Self as A>::Out;}impl A for u8{type Out=u16;}impl C for u8{fn get()->u32{73}}fn main(){}\n",
        false,
    ),
    (
        "generic-result",
        "trait A{type Out;}trait C:A{fn id(x:<Self as A>::Out)-><Self as A>::Out;}impl<T>A for [T;1]{type Out=T;}impl<T>C for [T;1]{fn id(x:T)->T{x}}fn main(){let _=<[u16;1]as C>::id(73);}\n",
        true,
    ),
    (
        "reverse-order",
        "trait A{type Out;}trait C:A{fn get()-><Self as A>::Out;}impl C for u8{fn get()->u16{73}}impl A for u8{type Out=u16;}fn main(){let _=<u8 as C>::get();}\n",
        true,
    ),
    (
        "trait-argument",
        "trait A<T>{type Out;}trait C:A<u16>{fn get()-><Self as A<u16>>::Out;}impl A<u16>for u8{type Out=u16;}impl C for u8{fn get()->u16{73}}fn main(){let _=<u8 as C>::get();}\n",
        true,
    ),
    (
        "wrong-trait-argument",
        "trait A<T>{type Out;}trait C:A<u16>{fn get()-><Self as A<u16>>::Out;}impl A<u16>for u8{type Out=u16;}impl A<u8>for u8{type Out=u32;}impl C for u8{fn get()-><Self as A<u8>>::Out{73}}fn main(){}\n",
        false,
    ),
    (
        "reference-input",
        "trait A{type Out;}trait C:A{fn id<'a>(x:&'a <Self as A>::Out)->&'a <Self as A>::Out;}impl A for u8{type Out=u16;}impl C for u8{fn id<'a>(x:&'a u16)->&'a u16{x}}fn main(){let x=73;let _=<u8 as C>::id(&x);}\n",
        true,
    ),
    (
        "method-bound",
        "trait Need{}impl Need for u32{}trait A{type Out;}trait C:A{fn get<T:Need>(x:T)-><Self as A>::Out;}impl A for u8{type Out=u16;}impl C for u8{fn get<T:Need>(_:T)->u16{73}}fn main(){let _=<u8 as C>::get(1u32);}\n",
        true,
    ),
    (
        "method-equality",
        "trait A{type Out;}trait C:A{fn get<T:A<Out=u16>>(x:<T as A>::Out)-><Self as A>::Out;}impl A for u8{type Out=u16;}impl C for u8{fn get<T:A<Out=u16>>(x:u16)->u16{x}}fn main(){let _=<u8 as C>::get::<u8>(73);}\n",
        true,
    ),
    (
        "rigid-parent-equality",
        "trait A{type Out;}trait C:A{fn id(x:<Self as A>::Out)-><Self as A>::Out;}impl A for u8{type Out=u16;}impl<T:A<Out=u16>>C for T{fn id(x:u16)->u16{x}}fn main(){let _=<u8 as C>::id(73);}\n",
        true,
    ),
    (
        "redundant-method-bound",
        "trait A{type Out;}trait C:A{fn get()-><Self as A>::Out where Self:A;}impl A for u8{type Out=u16;}impl C for u8{fn get()->u16{73}}fn main(){let _=<u8 as C>::get();}\n",
        true,
    ),
    (
        "captured-result",
        "trait A{type Out;}trait C:A{fn id(x:<Self as A>::Out)-><Self as A>::Out;}impl<'a>A for &'a u16{type Out=&'a u16;}impl<'a>C for &'a u16{fn id(x:&'a u16)->&'a u16{x}}fn main(){let x=73;let _=<&u16 as C>::id(&x);}\n",
        true,
    ),
    (
        "stronger-captured-result",
        "trait A{type Out;}trait C:A{fn id(x:<Self as A>::Out)-><Self as A>::Out;}impl<'a>A for &'a u16{type Out=&'a u16;}impl<'a>C for &'a u16{fn id(x:&'static u16)->&'static u16{x}}fn main(){}\n",
        false,
    ),
    (
        "invalid-supertrait",
        "trait A{type Out;}trait C:A<Out=u32>{fn get()-><Self as A>::Out;}impl A for u8{type Out=u16;}impl C for u8{fn get()->u32{73}}fn main(){}\n",
        false,
    ),
    (
        "own-and-inherited",
        "trait A{type Out;}trait C:A{type Own;fn get(x:<Self as A>::Out)-><Self as C>::Own;}impl A for u8{type Out=u16;}impl C for u8{type Own=u32;fn get(x:u16)->u32{x as u32}}fn main(){let _=<u8 as C>::get(73);}\n",
        true,
    ),
    (
        "redundant-generic-self",
        "trait A{type Out;}trait C:A{fn id(x:<Self as A>::Out)-><Self as A>::Out where Self:A;}impl<T>A for [T;1]{type Out=T;}impl<T>C for [T;1]{fn id(x:T)->T{x}}fn main(){let _=<[u16;1]as C>::id(73);}\n",
        false,
    ),
    (
        "redundant-transitive",
        "trait A{type Out;}trait B:A{}trait C:B{fn get()-><Self as A>::Out where Self:A;}impl A for u8{type Out=u16;}impl B for u8{}impl C for u8{fn get()->u16{73}}fn main(){let _=<u8 as C>::get();}\n",
        true,
    ),
    (
        "redundant-with-method-bound",
        "trait Need{}impl Need for u32{}trait A{type Out;}trait C:A{fn get<T:Need>(x:T)-><Self as A>::Out where Self:A;}impl A for u8{type Out=u16;}impl C for u8{fn get<T:Need>(_:T)->u16{73}}fn main(){let _=<u8 as C>::get(1u32);}\n",
        true,
    ),
    (
        "stronger-method-bound",
        "trait Need{}trait A{type Out;}trait C:A{fn get<T:Need>(x:T)-><Self as A>::Out where Self:A;}impl A for u8{type Out=u16;}impl C for u8{fn get<T:Need+Copy>(_:T)->u16{73}}fn main(){}\n",
        false,
    ),
    (
        "redundant-trait-parameter",
        "trait A{type Out;}trait C<T:Copy>:A{fn id<U:Clone>(x:T,y:U)-><Self as A>::Out where T:Copy;}impl A for u8{type Out=u16;}impl<T:Copy>C<T>for u8{fn id<U:Clone>(_:T,_:U)->u16{73}}fn main(){let _=<u8 as C<u32>>::id(1u32,2u8);}\n",
        true,
    ),
    (
        "distinct-method-application",
        "trait A<T>{type Out;}trait C:A<u8>{fn get<T>()-><Self as A<T>>::Out where Self:A<T>;}impl A<u8>for u16{type Out=u32;}impl A<u16>for u16{type Out=u64;}impl C for u16{fn get<T>()-><Self as A<T>>::Out where Self:A<T>{loop{}}}fn main(){}\n",
        true,
    ),
    (
        "wrong-method-application",
        "trait A<T>{type Out;}trait C:A<u8>{fn get<T>()-><Self as A<T>>::Out where Self:A<T>;}impl A<u8>for u16{type Out=u32;}impl A<u16>for u16{type Out=u64;}impl C for u16{fn get<T>()-><Self as A<u8>>::Out where Self:A<T>{73}}fn main(){}\n",
        false,
    ),
    (
        "redundant-captured-trait",
        "trait A<'a>{type Out;}trait C<'a>:A<'a>{fn id(x:<Self as A<'a>>::Out)-><Self as A<'a>>::Out where Self:A<'a>;}impl<'a>A<'a>for u8{type Out=&'a u16;}impl<'a>C<'a>for u8{fn id(x:&'a u16)->&'a u16{x}}fn main(){let x=73;let _=<u8 as C>::id(&x);}\n",
        false,
    ),
    (
        "stronger-captured-trait",
        "trait A<'a>{type Out;}trait C<'a>:A<'a>{fn id(x:<Self as A<'a>>::Out)-><Self as A<'a>>::Out where Self:A<'a>;}impl<'a>A<'a>for u8{type Out=&'a u16;}impl<'a>C<'a>for u8{fn id(x:&'static u16)->&'static u16{x}}fn main(){}\n",
        false,
    ),
    (
        "closed-function-bound",
        "trait A{type Out;}impl A for u8{type Out=u16;}fn f()-><u8 as A>::Out where u8:A{73}fn main(){let _=f();}\n",
        true,
    ),
    (
        "closed-trait-argument-bound",
        "trait A<T>{type Out;}impl A<u16>for u8{type Out=u32;}impl A<u8>for u8{type Out=u16;}fn f()-><u8 as A<u16>>::Out where u8:A<u16>{73}fn main(){let _=f();}\n",
        true,
    ),
    (
        "closed-explicit-equality",
        "trait A{type Out;}impl A for u8{type Out=u16;}fn f()-><u8 as A>::Out where u8:A<Out=u16>{73}fn main(){let _=f();}\n",
        true,
    ),
    (
        "wrong-closed-equality",
        "trait A{type Out;}impl A for u8{type Out=u16;}fn f()-><u8 as A>::Out where u8:A<Out=u32>{73}fn main(){}\n",
        false,
    ),
    (
        "open-trait-argument-bound",
        "trait A<T>{type Out;}impl A<u16>for u8{type Out=u16;}fn f<T>()-><u8 as A<T>>::Out where u8:A<T>{73u16}fn main(){}\n",
        false,
    ),
    (
        "wrong-closed-function-result",
        "trait A{type Out;}impl A for u8{type Out=u16;}fn f()-><u8 as A>::Out where u8:A{73u32}fn main(){}\n",
        false,
    ),
    (
        "closed-lifetime-implementation",
        "trait A{type Out;}impl<'a>A for &'a u16{type Out=&'a u16;}trait C:A{fn id(x:<Self as A>::Out)-><Self as A>::Out where Self:A;}impl C for &'static u16{fn id(x:&'static u16)->&'static u16{x}}static V:u16=73;fn main(){let _=<&'static u16 as C>::id(&V);}\n",
        true,
    ),
    (
        "closed-lifetime-escape",
        "trait A{type Out;}impl<'a>A for &'a u16{type Out=&'a u16;}trait C:A{fn id(x:<Self as A>::Out)-><Self as A>::Out where Self:A;}impl C for &'static u16{fn id(x:&'static u16)->&'static u16{x}}fn escape()->&'static u16{let x=73;<&'static u16 as C>::id(&x)}fn main(){let _=escape();}\n",
        false,
    ),
];
#[test]
fn inherited_signatures_match_independent_source_outcomes() -> Result<(), Box<dyn std::error::Error>>
{
    for &(name, source, accepted) in CASES {
        rustc::qualify(&format!("inherited-method-{name}"), source, accepted)?;
        let result = analyze(source);
        assert_eq!(result.is_ok(), accepted, "{name}: {result:?}");
    }
    Ok(())
}
fn analyze(source: &str) -> Result<(), String> {
    let (mut report, mut facts) = semantic::analyze(source)?;
    let entry = rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
    rustgm::lower::instances::build(&report, &mut facts, entry)
        .map(|_| ())
        .map_err(|e| e.to_string())
}
const NATIVE_CASES: &[(&str, &str, bool)] = &[
    (
        "closed-result",
        "trait A{type Out;}trait C:A{fn get()-><Self as A>::Out;}impl A for u8{type Out=u16;}impl C for u8{fn get()->u16{73}}fn main(){check(<u8 as C>::get()==73);}",
        false,
    ),
    (
        "generic-result",
        "trait A{type Out;}trait C:A{fn id(x:<Self as A>::Out)-><Self as A>::Out;}impl<T>A for [T;1]{type Out=T;}impl<T>C for [T;1]{fn id(x:T)->T{x}}fn main(){check(<[u16;1]as C>::id(73)==73);}",
        false,
    ),
    (
        "reverse-order",
        "trait A{type Out;}trait C:A{fn get()-><Self as A>::Out;}impl C for u8{fn get()->u16{73}}impl A for u8{type Out=u16;}fn main(){check(<u8 as C>::get()==73);}",
        false,
    ),
    (
        "trait-argument",
        "trait A<T>{type Out;}trait C:A<u16>{fn get()-><Self as A<u16>>::Out;}impl A<u16>for u8{type Out=u16;}impl C for u8{fn get()->u16{73}}fn main(){check(<u8 as C>::get()==73);}",
        false,
    ),
    (
        "reference-input",
        "trait A{type Out;}trait C:A{fn id<'a>(x:&'a <Self as A>::Out)->&'a <Self as A>::Out;}impl A for u8{type Out=u16;}impl C for u8{fn id<'a>(x:&'a u16)->&'a u16{x}}fn main(){let x=73;check(*<u8 as C>::id(&x)==73);}",
        false,
    ),
    (
        "method-bound",
        "trait Need{}impl Need for u32{}trait A{type Out;}trait C:A{fn get<T:Need>(x:T)-><Self as A>::Out;}impl A for u8{type Out=u16;}impl C for u8{fn get<T:Need>(_:T)->u16{73}}fn main(){check(<u8 as C>::get(1u32)==73);}",
        false,
    ),
    (
        "method-equality",
        "trait A{type Out;}trait C:A{fn get<T:A<Out=u16>>(x:<T as A>::Out)-><Self as A>::Out;}impl A for u8{type Out=u16;}impl C for u8{fn get<T:A<Out=u16>>(x:u16)->u16{x}}fn main(){check(<u8 as C>::get::<u8>(73)==73);}",
        false,
    ),
    (
        "rigid-parent-equality",
        "trait A{type Out;}trait C:A{fn id(x:<Self as A>::Out)-><Self as A>::Out;}impl A for u8{type Out=u16;}impl<T:A<Out=u16>>C for T{fn id(x:u16)->u16{x}}fn main(){check(<u8 as C>::id(73)==73);}",
        false,
    ),
    (
        "redundant-method-bound",
        "trait A{type Out;}trait C:A{fn get()-><Self as A>::Out where Self:A;}impl A for u8{type Out=u16;}impl C for u8{fn get()->u16{73}}fn main(){check(<u8 as C>::get()==73);}",
        false,
    ),
    (
        "captured-result",
        "trait A{type Out;}trait C:A{fn id(x:<Self as A>::Out)-><Self as A>::Out;}impl<'a>A for &'a u16{type Out=&'a u16;}impl<'a>C for &'a u16{fn id(x:&'a u16)->&'a u16{x}}fn main(){let x=73;check(*<&u16 as C>::id(&x)==73);}",
        false,
    ),
    (
        "own-and-inherited",
        "trait A{type Out;}trait C:A{type Own;fn get(x:<Self as A>::Out)-><Self as C>::Own;}impl A for u8{type Out=u16;}impl C for u8{type Own=u32;fn get(x:u16)->u32{x as u32}}fn main(){check(<u8 as C>::get(73)==73);}",
        false,
    ),
    (
        "redundant-transitive",
        "trait A{type Out;}trait B:A{}trait C:B{fn get()-><Self as A>::Out where Self:A;}impl A for u8{type Out=u16;}impl B for u8{}impl C for u8{fn get()->u16{73}}fn main(){check(<u8 as C>::get()==73);}",
        false,
    ),
    (
        "redundant-with-method-bound",
        "trait Need{}impl Need for u32{}trait A{type Out;}trait C:A{fn get<T:Need>(x:T)-><Self as A>::Out where Self:A;}impl A for u8{type Out=u16;}impl C for u8{fn get<T:Need>(_:T)->u16{73}}fn main(){check(<u8 as C>::get(1u32)==73);}",
        false,
    ),
    (
        "redundant-trait-parameter",
        "trait A{type Out;}trait C<T:Copy>:A{fn id<U:Clone>(x:T,y:U)-><Self as A>::Out where T:Copy;}impl A for u8{type Out=u16;}impl<T:Copy>C<T>for u8{fn id<U:Clone>(_:T,_:U)->u16{73}}fn main(){check(<u8 as C<u32>>::id(1u32,2u8)==73);}",
        false,
    ),
    (
        "closed-function-bound",
        "trait A{type Out;}impl A for u8{type Out=u16;}fn f()-><u8 as A>::Out where u8:A{73}fn main(){check(f()==73);}",
        false,
    ),
    (
        "closed-trait-argument-bound",
        "trait A<T>{type Out;}impl A<u16>for u8{type Out=u32;}impl A<u8>for u8{type Out=u16;}fn f()-><u8 as A<u16>>::Out where u8:A<u16>{73}fn main(){check(f()==73u32);}",
        false,
    ),
    (
        "closed-explicit-equality",
        "trait A{type Out;}impl A for u8{type Out=u16;}fn f()-><u8 as A>::Out where u8:A<Out=u16>{73}fn main(){check(f()==73);}",
        false,
    ),
    (
        "inherited-function-pointer",
        "trait A{type Out;}trait C:A{fn get()-><Self as A>::Out;}impl A for u8{type Out=u16;}impl C for u8{fn get()->u16{73}}fn main(){let f:fn()->u16=<u8 as C>::get;check(f()==73);}",
        false,
    ),
    (
        "inherited-dot-call",
        "trait A{type Out;}trait C:A{fn get(&self)-><Self as A>::Out;}impl A for u8{type Out=u16;}impl C for u8{fn get(&self)->u16{*self as u16}}fn main(){check(73u8.get()==73);}",
        false,
    ),
    (
        "closed-lifetime-implementation",
        "trait A{type Out;}impl<'a>A for &'a u16{type Out=&'a u16;}trait C:A{fn id(x:<Self as A>::Out)-><Self as A>::Out where Self:A;}impl C for &'static u16{fn id(x:&'static u16)->&'static u16{x}}static V:u16=73;fn main(){check(*<&'static u16 as C>::id(&V)==73);}",
        false,
    ),
];
#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn inherited_method_values_execute_against_rustc() -> Result<(), Box<dyn std::error::Error>> {
    native::check_cases("inherited-method-contracts", NATIVE_CASES)
}
