//! Declared outcomes distinguish marker bounds from storage and ordinary traits.
use crate::qualify as rustc;
use crate::semantic;

const CASES: &[(&str, &str, bool)] = &[
    ("auto-only", "fn f(_: &(dyn Send+Sync)){}", true),
    (
        "auto-only-conversion",
        "fn f(){let _: &dyn Send=&7u8;}",
        true,
    ),
    (
        "auto-only-unpin",
        "fn f<T:?Sized>(p:*const T){let _: &dyn Unpin=&p;}",
        true,
    ),
    (
        "auto-only-reject",
        "fn f(x:&*const u8){let _: &dyn Send=x;}",
        false,
    ),
    ("auto-only-repeat", "fn f(_: &(dyn Send+Send)){}", true),
    (
        "auto-only-drop",
        "fn f(x:&(dyn Send+Sync))->&dyn Send{x}",
        true,
    ),
    (
        "auto-only-add",
        "fn f(x:&dyn Send)->&(dyn Send+Sync){x}",
        false,
    ),
    (
        "scalar",
        "fn need<T:Send+Sync+Unpin>(){}fn f(){need::<u8>();}",
        true,
    ),
    (
        "str",
        "fn need<T:?Sized+Send+Sync>(){}fn f(){need::<str>();}",
        true,
    ),
    (
        "raw-send",
        "fn need<T:Send>(){}fn f(){need::<*const u8>();}",
        false,
    ),
    (
        "raw-sync",
        "fn need<T:Sync>(){}fn f(){need::<*mut u8>();}",
        false,
    ),
    (
        "raw-unpin",
        "fn need<T:Unpin>(){}fn f(){need::<*const u8>();}",
        true,
    ),
    (
        "generic",
        "fn need<T:Send>(){}fn f<T:Send>(){need::<T>();}",
        true,
    ),
    (
        "unconstrained",
        "fn need<T:Send>(){}fn f<T>(){need::<T>();}",
        false,
    ),
    (
        "no-implied-sized",
        "fn sized<T>(){}fn f<T:?Sized+Send>(){sized::<T>();}",
        false,
    ),
    (
        "no-implied-copy",
        "fn need<T:Copy>(){}fn f<T:Send>(){need::<T>();}",
        false,
    ),
    (
        "shared",
        "fn need<T:Send>(){}fn f<T:Sync>(){need::<&T>();}",
        true,
    ),
    (
        "shared-wrong",
        "fn need<T:Send>(){}fn f<T:Send>(){need::<&T>();}",
        false,
    ),
    (
        "mutable",
        "fn need<T:Send>(){}fn f<T:Send>(){need::<&mut T>();}",
        true,
    ),
    (
        "reference-unpin",
        "fn need<T:Unpin>(){}fn f<T:?Sized>(){need::<&mut T>();}",
        true,
    ),
    (
        "unwind",
        "fn need<T:core::panic::UnwindSafe>(){}fn f(){need::<&u8>();}",
        true,
    ),
    (
        "mutable-unwind",
        "fn need<T:core::panic::UnwindSafe>(){}fn f(){need::<&mut u8>();}",
        false,
    ),
    (
        "raw-unwind",
        "fn need<T:core::panic::UnwindSafe>(){}fn f<T:core::panic::RefUnwindSafe>(){need::<*const T>();}",
        true,
    ),
    (
        "panic-not-prelude",
        "use core::panic as p;fn f<T:UnwindSafe>(){}",
        false,
    ),
    (
        "panic-alias",
        "use core::panic::UnwindSafe as Safe;fn need<T:Safe>(){}fn f(){need::<u8>();}",
        true,
    ),
    (
        "module-alias",
        "use core::marker as m;fn need<T:m::Send>(){}fn f(){need::<u8>();}",
        true,
    ),
    (
        "shadowed",
        "trait Send{}fn need<T:Send>(){}fn f(){need::<u8>();}",
        false,
    ),
    (
        "shadowed-impl",
        "trait Send{}impl Send for u8{}fn need<T:Send>(){}fn f(){need::<u8>();}",
        true,
    ),
    (
        "fields",
        "struct W<T>(T);fn need<T:Send>(){}fn f<T:Send>(){need::<W<T>>();}",
        true,
    ),
    (
        "bad-field",
        "struct W(*const u8);fn need<T:Send>(){}fn f(){need::<W>();}",
        false,
    ),
    (
        "empty-array",
        "fn need<T:Send>(){}fn f(){need::<[*const u8;0]>();}",
        false,
    ),
    (
        "function-input",
        "struct W<T>(fn(T));fn need<T:Send>(){}fn f<T>(){need::<W<T>>();}",
        true,
    ),
    (
        "recursive",
        "struct A<'a>(&'a B<'a>);struct B<'a>(&'a A<'a>);fn need<T:Send>(){}fn f(){need::<A<'static>>();}",
        true,
    ),
    (
        "recursive-bad",
        "struct A<'a>(&'a B<'a>);struct B<'a>(&'a A<'a>,*const u8);fn need<T:Send>(){}fn f(){need::<A<'static>>();}",
        false,
    ),
    (
        "projection",
        "trait A{type V:Send;}fn need<T:Send>(){}fn f<T:A>(){need::<T::V>();}",
        true,
    ),
    (
        "projected-fields",
        "trait A{type V:Send;}struct W<T:A>(T::V);fn need<T:Send>(){}fn f<T:A>(){need::<W<T>>();}",
        true,
    ),
    (
        "projected-bad",
        "trait A{type V;}struct W<T:A>(T::V);fn need<T:Send>(){}fn f<T:A>(){need::<W<T>>();}",
        false,
    ),
    (
        "opaque",
        "fn make()->impl Send{7u8}fn need<T:Send>(_:T){}fn f(){need(make());}",
        true,
    ),
    (
        "opaque-bad",
        "fn make()->impl Send{0usize as *const u8}",
        false,
    ),
    (
        "object",
        "trait A{}impl A for u8{}fn f(){let _: &(dyn A+Send+Sync)=&7u8;}",
        true,
    ),
    (
        "object-reject",
        "trait A{}struct W(*const u8);impl A for W{}fn f(x:&W){let _: &(dyn A+Send)=x;}",
        false,
    ),
    (
        "object-permission",
        "trait A{}fn need<T:?Sized+Send>(){}fn f(){need::<dyn A+Send>();}",
        true,
    ),
    (
        "object-no-permission",
        "trait A{}fn need<T:?Sized+Send>(){}fn f(){need::<dyn A>();}",
        false,
    ),
    (
        "inherited",
        "trait A:Send{}fn need<T:?Sized+Send>(){}fn f(){need::<dyn A>();}",
        true,
    ),
    (
        "unrelated",
        "trait A<T:Send>{}fn need<T:?Sized+Send>(){}fn f(){need::<dyn A<u8>>();}",
        false,
    ),
    (
        "impl-contract",
        "trait A:Send{}struct W(*const u8);impl A for W{}",
        false,
    ),
];

#[test]
fn auto_trait_sources_match_pinned_rustc() -> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, source, accepted) in CASES {
        rustc::qualify(&format!("auto-{name}"), source, accepted)?;
        let result = semantic::analyze(source);
        if result.is_ok() != accepted {
            failures.push(format!("{name}: {:?}", result.err()));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
