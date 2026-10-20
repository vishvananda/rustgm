//! Explicit auto impls replace structural defaults for the entire nominal family.
#[path = "support/verified_regions.rs"]
mod regions;
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;

const CASES: &[(&str, &str, bool)] = &[
    (
        "opaque-pattern-scalar-missing",
        "fn main(){need(make());}fn need<T:Send>(_:T){}trait A{}impl A for u8{}const C:u8=1;fn make()->impl A{match 1u8{C=>7u8}}",
        false,
    ),
    (
        "opaque-pattern-scalar-complete",
        "fn main(){need(make());}fn need<T:Send>(_:T){}trait A{}impl A for u8{}const C:u8=1;fn make()->impl A{match 1u8{C=>7u8,_=>9}}",
        true,
    ),
    (
        "opaque-pattern-tuple-missing",
        "fn main(){need(make());}fn need<T:Send>(_:T){}trait A{}impl A for u8{}const C:(u8,u8)=(1,2);fn make()->impl A{match (1u8,2u8){C=>7u8}}",
        false,
    ),
    (
        "opaque-pattern-tuple-complete",
        "fn main(){need(make());}fn need<T:Send>(_:T){}trait A{}impl A for u8{}const C:(u8,u8)=(1,2);fn make()->impl A{match (1u8,2u8){C=>7u8,_=>9}}",
        true,
    ),
    (
        "opaque-return-marker",
        "trait A{}impl A for u8{}fn outer()->impl A+Send{inner()}fn inner()->impl A{7u8}",
        true,
    ),
    (
        "opaque-return-supertrait",
        "trait A{}impl A for u8{}trait B:Send{}impl<T:Send>B for T{}fn outer()->impl B{inner()}fn inner()->impl A{7u8}",
        true,
    ),
    (
        "opaque-return-marker-reversed",
        "trait A{}impl A for u8{}fn inner()->impl A{7u8}fn outer()->impl A+Send{inner()}",
        true,
    ),
    (
        "opaque-nominal-constructor",
        "fn f(){let _=W(make());}struct W<T:Send>(T);trait A{}impl A for u8{}fn make()->impl A{7u8}",
        true,
    ),
    (
        "opaque-nominal-field",
        "fn f(){let w=W(make());let _=w.0;}struct W<T:Send>(T);trait A{}impl A for u8{}fn make()->impl A{7u8}",
        true,
    ),
    (
        "opaque-impl-condition",
        "fn f(){let _=need(make());}fn need<T:B>(_:T){}trait A{}trait B{}impl<T:Send>B for T{}impl A for u8{}fn make()->impl A{7u8}",
        true,
    ),
    (
        "opaque-return-quantified",
        "trait A{}impl A for u8{}trait B<'a>{}impl<'a,T:Send>B<'a> for T{}fn outer()->impl for<'a>B<'a>{inner()}fn inner()->impl A{7u8}",
        true,
    ),
    (
        "opaque-return-quantified-unpaid",
        "trait A{}impl A for u8{}trait B<'a>{}impl<'a:'static,T:Send>B<'a> for T{}fn outer()->impl for<'a>B<'a>{inner()}fn inner()->impl A{7u8}",
        false,
    ),
    (
        "opaque-nominal-quantified",
        "fn f(){let _=W(make());}struct W<T:for<'a>B<'a>>(T);trait A{}impl A for u8{}trait B<'a>{}impl<'a,T:Send>B<'a> for T{}fn make()->impl A{7u8}",
        true,
    ),
    (
        "opaque-nominal-quantified-unpaid",
        "fn f(){let _=W(make());}struct W<T:for<'a>B<'a>>(T);trait A{}impl A for u8{}trait B<'a>{}impl<'a:'static,T:Send>B<'a> for T{}fn make()->impl A{7u8}",
        false,
    ),
    (
        "opaque-return-marker-fails",
        "trait A{}struct P(*const u8);impl A for P{}fn outer(x:*const u8)->impl A+Send{inner(x)}fn inner(x:*const u8)->impl A{P(x)}",
        false,
    ),
    (
        "opaque-return-supertrait-fails",
        "trait A{}struct P(*const u8);impl A for P{}trait B:Send{}impl<T:Send>B for T{}fn outer(x:*const u8)->impl B{inner(x)}fn inner(x:*const u8)->impl A{P(x)}",
        false,
    ),
    (
        "opaque-nominal-fails",
        "fn f(x:*const u8){let _=W(make(x));}struct W<T:Send>(T);trait A{}struct P(*const u8);impl A for P{}fn make(x:*const u8)->impl A{P(x)}",
        false,
    ),
    (
        "opaque-self-auto",
        "trait A{}impl A for u8{}fn need<T:Send>(_:T){}fn make()->impl A{need(make());7u8}",
        false,
    ),
    (
        "opaque-mutual-auto",
        "trait A{}impl A for u8{}fn need<T:Send>(_:T){}fn a()->impl A{need(b());7u8}fn b()->impl A{need(a());7u8}",
        false,
    ),
    (
        "opaque-declared-self",
        "trait A{}impl A for u8{}fn need<T:Send>(_:T){}fn make()->impl A+Send{need(make());7u8}",
        true,
    ),
    (
        "opaque-indirect-declaration",
        "trait A{}impl A for u8{}fn need<T:Send>(_:T){}fn a()->impl A+Send{need(b());7u8}fn b()->impl A{need(a());7u8}",
        true,
    ),
    (
        "opaque-nested-definitions",
        "fn main(){need(make0());}fn need<T:Send>(_:T){}trait A{}impl A for u8{}fn make0()->impl A{need(make1());7u8}fn make1()->impl A{need(make2());7u8}fn make2()->impl A{7u8}",
        true,
    ),
    (
        "opaque-plain",
        "trait A{}impl A for u8{}fn make()->impl A{7u8}fn need<T:Send>(_:T){}fn f(){need(make());}",
        true,
    ),
    (
        "opaque-generic-positive",
        "trait A{}struct W<T>(T);impl<T>A for W<T>{}fn make<T>(x:T)->impl A{W(x)}fn need<T:Send>(_:T){}fn f(){need(make(7u8));}",
        true,
    ),
    (
        "opaque-generic-negative",
        "trait A{}struct W<T>(T);impl<T>A for W<T>{}fn make<T>(x:T)->impl A{W(x)}fn need<T:Send>(_:T){}fn f(x:*const u8){need(make(x));}",
        false,
    ),
    (
        "opaque-caller-positive",
        "trait A{}struct W<T>(T);impl<T>A for W<T>{}fn make<T>(x:T)->impl A{W(x)}fn need<T:Send>(_:T){}fn f<T:Send>(x:T){need(make(x));}",
        true,
    ),
    (
        "opaque-caller-negative",
        "trait A{}struct W<T>(T);impl<T>A for W<T>{}fn make<T>(x:T)->impl A{W(x)}fn need<T:Send>(_:T){}fn f<T>(x:T){need(make(x));}",
        false,
    ),
    (
        "opaque-manual",
        "trait A{}struct W(*const u8);impl A for W{}unsafe impl Send for W{}fn make(x:*const u8)->impl A{W(x)}fn need<T:Send>(_:T){}fn f(x:*const u8){need(make(x));}",
        true,
    ),
    (
        "opaque-reversed",
        "fn f(){need(make());}fn need<T:Send>(_:T){}fn make()->impl A{7u8}trait A{}impl A for u8{}",
        true,
    ),
    (
        "opaque-ordinary-private",
        "trait A{}impl A for u8{}fn make()->impl A{7u8}fn need<T:Copy>(_:T){}fn f(){need(make());}",
        false,
    ),
    (
        "opaque-chain",
        "trait A{}impl A for u8{}fn a()->impl A{b()}fn b()->impl A{7u8}fn need<T:Send>(_:T){}fn f(){need(a());}",
        true,
    ),
    (
        "cycle-unused-binder",
        "struct W;impl Unpin for W where for<'a> W:Unpin{}fn need<T:Unpin>(){}fn f(){need::<W>();}",
        true,
    ),
    (
        "cycle-generic-binder",
        "struct W<T>(T);impl<T> Unpin for W<T> where for<'a> W<T>:Unpin{}fn need<T:Unpin>(){}fn f(){need::<W<u8>>();}",
        true,
    ),
    (
        "cycle-quantified-reference",
        "struct W;unsafe impl Sync for W where for<'a> &'a W:Sync{}fn need<T:Sync>(){}fn f(){need::<W>();}",
        true,
    ),
    (
        "cycle-generic-reference",
        "struct W<T>(T);unsafe impl<T> Sync for W<T> where for<'a> &'a W<T>:Sync{}fn need<T:Sync>(){}fn f(){need::<W<*const u8>>();}",
        true,
    ),
    (
        "cycle-private-witnesses",
        "struct A<'a>(&'a u8);impl<'a> Unpin for A<'a> where A<'static>:Unpin{}struct C(A<'static>,A<'static>);fn need<T:Unpin>(){}fn f(){need::<C>();}",
        true,
    ),
    (
        "quantified-static-context",
        "struct W<'a>(&'a u8);unsafe impl<'a:'static> Sync for W<'a>{}struct V;impl Unpin for V where for<'a> W<'static>:Sync{}fn need<T:Unpin>(){}fn f(){need::<V>();}",
        true,
    ),
    (
        "quantified-local-context-fails",
        "struct W<'a>(&'a u8);unsafe impl<'a:'static> Sync for W<'a>{}struct V;impl Unpin for V where for<'a> W<'a>:Sync{}fn need<T:Unpin>(){}fn f(){need::<V>();}",
        false,
    ),
    (
        "early-alias-header",
        "struct W(*const u8);trait A{type Out;}impl<T:Send> A for T{type Out=u8;}type Output=<W as A>::Out;type Alias=W;unsafe impl Send for Alias{}fn f(_:Output){}",
        true,
    ),
    (
        "quantified-condition",
        "struct W<T>(T);impl<T> Unpin for W<T> where for<'a> &'a T:Send{}fn need<T:Unpin>(){}fn f<T:Sync>(){need::<W<T>>();}",
        true,
    ),
    (
        "quantified-condition-fails",
        "struct W<T>(T);impl<T> Unpin for W<T> where for<'a> &'a T:Send{}fn need<T:Unpin>(){}fn f<T:Send>(){need::<W<T>>();}",
        false,
    ),
    (
        "object-method-missing-marker",
        "trait A{fn f(&self)->u8 where Self:Send{7}}struct W;unsafe impl Send for W{}impl A for W{}fn call(p:&dyn A)->u8{p.f()}",
        false,
    ),
    (
        "cyclic-cache-positive",
        "struct A<T>(fn(T));struct B<T>(fn(T));struct C<T>(A<T>,B<T>);impl<T> Unpin for A<T> where B<T>:Unpin,T:Sync{}impl<T> Unpin for B<T> where A<T>:Unpin{}fn need<T:Unpin>(){}fn f(){need::<C<u8>>();}",
        true,
    ),
    (
        "cyclic-cache-negative",
        "struct A<T>(fn(T));struct B<T>(fn(T));struct C<T>(A<T>,B<T>);impl<T> Unpin for A<T> where B<T>:Unpin,T:Sync{}impl<T> Unpin for B<T> where A<T>:Unpin{}fn need<T:Unpin>(){}fn f(){need::<C<*const u8>>();}",
        false,
    ),
    (
        "cycle-self",
        "struct W;impl Unpin for W where W:Unpin{}fn need<T:Unpin>(){}fn f(){need::<W>();}",
        true,
    ),
    (
        "cycle-pair",
        "struct W;struct V;impl Unpin for W where V:Unpin{}impl Unpin for V where W:Unpin{}fn need<T:Unpin>(){}fn f(){need::<W>();}",
        true,
    ),
    (
        "cycle-mixed",
        "trait A{}struct W;impl A for W where W:Unpin{}impl Unpin for W where W:A{}fn need<T:Unpin>(){}fn f(){need::<W>();}",
        false,
    ),
    (
        "cycle-field",
        "struct W;struct V(W);impl Unpin for W where V:Unpin{}fn need<T:Unpin>(){}fn f(){need::<W>();}",
        true,
    ),
    (
        "cycle-conditional",
        "struct W<T>(T);impl<T> Unpin for W<T> where W<T>:Unpin{}fn need<T:Unpin>(){}fn f(){need::<W<u8>>();}",
        true,
    ),
    (
        "cycle-conditional-bad",
        "struct W<T>(T);impl<T:Sync> Unpin for W<T> where W<T>:Unpin{}fn need<T:Unpin>(){}fn f(){need::<W<*const u8>>();}",
        false,
    ),
    (
        "cycle-lifetime-self",
        "struct W<'a>(&'a u8);impl<'a> Unpin for W<'a> where W<'a>:Unpin{}fn need<T:Unpin>(){}fn f<'a>(){need::<W<'a>>();}",
        false,
    ),
    (
        "cycle-lifetime-static",
        "struct W<'a>(&'a u8);impl<'a> Unpin for W<'a> where W<'static>:Unpin{}fn need<T:Unpin>(){}fn f<'a>(){need::<W<'a>>();}",
        true,
    ),
    (
        "cycle-swap",
        "struct W<'a,'b>(&'a u8,&'b u8);impl<'a,'b> Unpin for W<'a,'b> where W<'b,'a>:Unpin{}fn need<T:Unpin>(){}fn f<'a,'b>(){need::<W<'a,'b>>();}",
        false,
    ),
    (
        "cycle-swap-bad",
        "struct W<'a,'b>(&'a u8,&'b u8);impl<'a,'b> Unpin for W<'a,'b> where 'a:'b,W<'b,'a>:Unpin{}fn need<T:Unpin>(){}fn f<'a,'b>(){need::<W<'a,'b>>();}",
        false,
    ),
    (
        "cycle-swap-equal",
        "struct W<'a,'b>(&'a u8,&'b u8);impl<'a,'b> Unpin for W<'a,'b> where 'a:'b,W<'b,'a>:Unpin{}fn need<T:Unpin>(){}fn f<'a:'b,'b:'a>(){need::<W<'a,'b>>();}",
        false,
    ),
    (
        "send",
        "struct W(*const u8);unsafe impl Send for W{}fn need<T:Send>(){}fn f(){need::<W>();}",
        true,
    ),
    (
        "sync",
        "struct W(*mut u8);unsafe impl Sync for W{}fn need<T:Sync>(){}fn f(){need::<W>();}",
        true,
    ),
    ("safe-send", "struct W;impl Send for W{}", false),
    ("safe-sync", "struct W;impl Sync for W{}", false),
    ("unsafe-unpin", "struct W;unsafe impl Unpin for W{}", false),
    (
        "unwind",
        "struct W<'a>(&'a mut u8);impl<'a> core::panic::UnwindSafe for W<'a>{}fn need<T:core::panic::UnwindSafe>(){}fn f<'a>(){need::<W<'a>>();}",
        true,
    ),
    (
        "generic",
        "struct W<T>(T);unsafe impl<T> Send for W<T>{}fn need<T:Send>(){}fn f<T>(){need::<W<T>>();}",
        true,
    ),
    (
        "condition",
        "trait A{}impl A for u8{}struct W<T>(T);impl<T:A> Unpin for W<T>{}fn need<T:Unpin>(){}fn f(){need::<W<u8>>();}",
        true,
    ),
    (
        "condition-fails",
        "trait A{}struct W<T>(T);impl<T:A> Unpin for W<T>{}fn need<T:Unpin>(){}fn f(){need::<W<u8>>();}",
        false,
    ),
    (
        "condition-param",
        "trait A{}struct W<T>(T);impl<T:A> Unpin for W<T>{}fn need<T:Unpin>(){}fn f<T:A>(){need::<W<T>>();}",
        true,
    ),
    (
        "marker-condition",
        "struct W<T>(T);impl<T:Sync> Unpin for W<T>{}fn need<T:Unpin>(){}fn f<T:Sync>(){need::<W<T>>();}",
        true,
    ),
    (
        "marker-condition-fails",
        "struct W<T>(T);impl<T:Sync> Unpin for W<T>{}fn need<T:Unpin>(){}fn f(){need::<W<*const u8>>();}",
        false,
    ),
    (
        "specialized",
        "struct W<T>(T);impl Unpin for W<u8>{}fn need<T:Unpin>(){}fn f(){need::<W<u8>>();}",
        true,
    ),
    (
        "specialized-fails",
        "struct W<T>(T);impl Unpin for W<u8>{}fn need<T:Unpin>(){}fn f(){need::<W<u16>>();}",
        false,
    ),
    (
        "disjoint",
        "struct W<T>(T);impl Unpin for W<u8>{}impl Unpin for W<u16>{}fn need<T:Unpin>(){}fn f(){need::<W<u8>>();need::<W<u16>>();}",
        true,
    ),
    (
        "overlap",
        "struct W<T>(T);impl<T> Unpin for W<T>{}impl Unpin for W<u8>{}",
        false,
    ),
    (
        "nested",
        "struct W(*const u8);struct Outer(W);unsafe impl Send for W{}fn need<T:Send>(){}fn f(){need::<Outer>();}",
        true,
    ),
    (
        "nested-fails",
        "struct W<T>(T);struct Outer(W<u16>);impl Unpin for W<u8>{}fn need<T:Unpin>(){}fn f(){need::<Outer>();}",
        false,
    ),
    (
        "alias",
        "struct W(*const u8);type Alias=W;unsafe impl Send for Alias{}fn need<T:Send>(){}fn f(){need::<W>();}",
        true,
    ),
    (
        "unrelated-alias",
        "struct W;struct V;type Alias=V;impl Unpin for Alias{}fn need<T:Unpin>(){}fn f(){need::<W>();}",
        true,
    ),
    (
        "source-order",
        "fn need<T:Send>(){}fn f(){need::<W>();}unsafe impl Send for W{}struct W(*const u8);",
        true,
    ),
    (
        "const-specialized",
        "struct W<const N:usize>;impl Unpin for W<3>{}fn need<T:Unpin>(){}fn f(){need::<W<3>>();}",
        true,
    ),
    (
        "const-specialized-fails",
        "struct W<const N:usize>;impl Unpin for W<3>{}fn need<T:Unpin>(){}fn f(){need::<W<4>>();}",
        false,
    ),
    (
        "lifetime",
        "struct W<'a>(&'a u8);unsafe impl<'a:'static> Send for W<'a>{}fn need<T:Send>(_:T){}fn f(x:W<'static>){need(x);}",
        true,
    ),
    (
        "lifetime-fails",
        "struct W<'a>(&'a u8);unsafe impl<'a:'static> Send for W<'a>{}fn need<T:Send>(_:T){}fn f<'a>(x:W<'a>){need(x);}",
        false,
    ),
    (
        "whole-premise",
        "trait A{}struct W<T>(T);impl<T:A> Unpin for W<T>{}fn need<T:Unpin>(){}fn f<T>() where W<T>:Unpin{need::<W<T>>();}",
        true,
    ),
    ("reference", "struct W;impl Unpin for &W{}", false),
    ("tuple", "struct W;impl Unpin for (W,){}", false),
    ("array", "struct W;impl Unpin for [W;1]{}", false),
    ("dynamic", "trait A{}impl Unpin for dyn A{}", false),
    ("blanket", "impl<T> Unpin for T{}", false),
    ("arguments", "struct W;impl Unpin<u8> for W{}", false),
    ("member", "struct W;impl Unpin for W{fn f(){}}", false),
    ("unconstrained", "struct W;impl<T> Unpin for W{}", false),
    ("negative", "struct W;impl !Unpin for W{}", false),
    (
        "object",
        "struct W(*const u8);unsafe impl Send for W{}fn f(x:&W){let _: &dyn Send=x;}",
        true,
    ),
];

#[test]
fn explicit_auto_sources_match_pinned_rustc() -> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, source, accepted) in CASES {
        rustc::qualify(&format!("explicit-marker-{name}"), source, accepted)?;
        let result = regions::prove(source);
        if result.is_ok() != accepted {
            failures.push(format!("{name}: {:?}", result.err()));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
