//! Published associated results retain their input formation and implementation obligations.
use crate::native_references as native;
use crate::qualify as rustc;
use crate::semantic;
const CASES: &[(&str, &str, bool)] = &[
    (
        "structured-later-read",
        "struct W<T>(T);trait A{type Out;}impl<T:Copy>A for W<T>{type Out=u8;}type P<T>=(W<T>,<W<T>as A>::Out);fn main(){let p:P<_>=(W(0),7u8);let a=p.1;let _:W<u16>=p.0;let _:u8=a;}",
        true,
    ),
    (
        "erased-result-invalid-copy",
        "struct W<T>(T);trait A{type Out;}impl<T:Copy>A for W<T>{type Out=u8;}type P<T>=(W<T>,<W<T>as A>::Out);fn main(){let mut x=0u8;let p:P<_>=(W((&mut x,0)),7u8);let a=p.1;let _:W<(&mut u8,u16)>=p.0;let _:u8=a;}",
        false,
    ),
    (
        "ambiguous-premise-later-valid",
        "struct W<T>(T);trait B{}impl B for u8{}impl B for u16{}trait A{type Out;}impl<T:B>A for W<T>{type Out=u8;}type P<T>=(W<T>,<W<T>as A>::Out);fn main(){let p:P<_>=(W(0),7u8);let a=p.1;let _:W<u16>=p.0;let _:u8=a;}",
        true,
    ),
    (
        "erased-result-invalid-trait",
        "struct W<T>(T);trait B{}impl B for u8{}impl B for u16{}trait A{type Out;}impl<T:B>A for W<T>{type Out=u8;}type P<T>=(W<T>,<W<T>as A>::Out);fn main(){let p:P<_>=(W(0),7u8);let a=p.1;let _:W<u32>=p.0;let _:u8=a;}",
        false,
    ),
    (
        "known-head-erased-unbound-type",
        "struct W<T>(T);trait A{type Out;}impl<T>A for W<T>{type Out=u8;}type Q<T>=<W<T>as A>::Out;fn main(){let _:Q<_>=7u8;}",
        false,
    ),
    (
        "known-head-identity-output",
        "struct W<T>(T);trait A{type Out;}impl<T:Copy>A for W<T>{type Out=T;}type Q<T>=<W<T>as A>::Out;fn main(){let _:Q<_>=7u8;}",
        true,
    ),
    (
        "known-head-shared-output",
        "struct W<T>(T);trait A{type Out;}impl<T:Copy>A for W<T>{type Out=(T,T);}type Q<T>=<W<T>as A>::Out;fn main(){let _:Q<_>=(7u8,9u8);}",
        true,
    ),
    (
        "known-head-shared-output-conflict",
        "struct W<T>(T);trait A{type Out;}impl<T:Copy>A for W<T>{type Out=(T,T);}type Q<T>=<W<T>as A>::Out;fn main(){let _:Q<_>=(7u8,9u16);}",
        false,
    ),
    (
        "known-head-const-output",
        "struct W<const N:usize>;trait A{type Out;}impl<const N:usize>A for W<N>{type Out=[u8;N];}type Q<const N:usize>=<W<N>as A>::Out;fn main(){let _:Q<_>=[7u8,9u8];}",
        true,
    ),
    (
        "known-head-erased-unbound-const",
        "struct W<const N:usize>;trait A{type Out;}impl<const N:usize>A for W<N>{type Out=u8;}type Q<const N:usize>=<W<N>as A>::Out;fn main(){let _:Q<_>=7u8;}",
        false,
    ),
    (
        "erased-result-valid-outlives",
        "struct W<T>(T);trait A<'a>{type Out;}impl<'a,T:'a>A<'a>for W<T>{type Out=&'a u8;}type P<'a,T>=(W<T>,<W<T>as A<'a>>::Out);static V:u8=7;fn f<'a>(x:&'a u8){let p:P<'a,_>=(W((x,0)),&V);let _:u8=*p.1;let _:W<(&'a u8,u16)>=p.0;}fn main(){}",
        true,
    ),
    (
        "erased-result-invalid-outlives",
        "struct W<T>(T);trait A<'a>{type Out;}impl<'a,T:'a>A<'a>for W<T>{type Out=&'a u8;}type P<'a,T>=(W<T>,<W<T>as A<'a>>::Out);static V:u8=7;fn f<'a>(x:&'a u8){let p:P<'static,_>=(W((x,0)),&V);let _:u8=*p.1;let _:W<(&'a u8,u16)>=p.0;}fn main(){}",
        false,
    ),
    (
        "ordinary-erased-const-alias",
        "type E<const N:usize>=u8;fn main(){let _:E<_>=7u8;}",
        true,
    ),
];
const CONTEXTS: &[(&str, &str, bool)] = &[
    (
        "known-before-read",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type P<T>=(T,<T as A>::Out);fn main(){let p:P<_>=(0u16,7u8);let a=p.1;let _:u8=a;}",
        true,
    ),
    (
        "later-input-read",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type P<T>=(T,<T as A>::Out);fn main(){let p:P<_>=(0,7u8);let a=p.1;let _:u16=p.0;let _:u8=a;}",
        true,
    ),
    (
        "fallback-input-read",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type P<T>=(T,<T as A>::Out);fn main(){let p:P<_>=(0,7u8);let a=p.1;let _:u8=a;}",
        true,
    ),
    (
        "later-input-explicit-output",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type P<T>=(T,<T as A>::Out);fn main(){let p:P<_>=(0,7u8);let a:u8=p.1;let _:u16=p.0;}",
        true,
    ),
    (
        "later-input-wrong-output",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type P<T>=(T,<T as A>::Out);fn main(){let p:P<_>=(0,7u8);let a:u32=p.1;let _:u16=p.0;}",
        false,
    ),
    (
        "later-input-comparison",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type P<T>=(T,<T as A>::Out);fn main(){let p:P<_>=(0,7u8);let yes=p.1==7u8;let _:u16=p.0;}",
        true,
    ),
    (
        "later-input-arithmetic",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type P<T>=(T,<T as A>::Out);fn main(){let p:P<_>=(0,7u8);let a=p.1+3u8;let _:u16=p.0;}",
        true,
    ),
    (
        "later-input-call-argument",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type P<T>=(T,<T as A>::Out);fn take(_:u8){}fn main(){let p:P<_>=(0,7u8);take(p.1);let _:u16=p.0;}",
        true,
    ),
    (
        "later-input-aggregate-copy",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type P<T>=(T,<T as A>::Out);fn main(){let p:P<_>=(0,7u8);let q=p;let _:u16=q.0;}",
        true,
    ),
    (
        "unconstrained-input",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type P<T>=(T,<T as A>::Out);fn main(){let p:P<_>=loop{};let a:u8=p.1;}",
        false,
    ),
    (
        "later-input-function-result",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=fn(u8);}type P<T>=(T,<T as A>::Out);fn g(_:u8){}fn main(){let p:P<_>=(0,g as fn(u8));let h:fn(u8)=p.1;let _:u16=p.0;h(7);}",
        true,
    ),
    (
        "later-input-captured-result",
        "trait A{type Out;}impl<'a,T:Copy>A for (&'a u8,T){type Out=&'a u8;}type P<'a,T>=(T,<(&'a u8,T) as A>::Out);fn f<'a>(x:&'a u8)->&'a u8{let p:P<'a,_>=(0,x);let a=p.1;let _:u16=p.0;a}fn main(){}",
        true,
    ),
    (
        "later-input-captured-escape",
        "trait A{type Out;}impl<'a,T:Copy>A for (&'a u8,T){type Out=&'a u8;}type P<'a,T>=(T,<(&'a u8,T) as A>::Out);fn f<'a>(x:&'a u8)->&'static u8{let p:P<'a,_>=(0,x);let a=p.1;let _:u16=p.0;a}fn main(){}",
        false,
    ),
    (
        "later-implicit-function-pointer",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=fn(u8);}type P<T>=(T,<T as A>::Out);fn g(_:u8){}fn main(){let p:P<_>=(0,g);let _:u16=p.0;(p.1)(7);}",
        true,
    ),
    (
        "reject-later-function-signature",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=fn(u8);}type P<T>=(T,<T as A>::Out);fn g(_:u16){}fn main(){let p:P<_>=(0,g);let _:u16=p.0;}",
        false,
    ),
    (
        "later-shared-reborrow",
        "trait A<'a>{type Out;}impl<'a,T:Copy>A<'a>for T{type Out=&'a u8;}type P<'a,T>=(T,<T as A<'a>>::Out);fn f<'a>(x:&'a mut u8)->u8{let p:P<'_,_>=(0,x);let _:u16=p.0;*p.1}fn main(){}",
        true,
    ),
    (
        "reject-later-shared-reborrow-write",
        "trait A<'a>{type Out;}impl<'a,T:Copy>A<'a>for T{type Out=&'a u8;}type P<'a,T>=(T,<T as A<'a>>::Out);fn f<'a>(x:&'a mut u8)->u8{let p:P<'_,_>=(0,x);let _:u16=p.0;*x=73;*p.1}fn main(){}",
        false,
    ),
    (
        "later-identity-reborrow",
        "trait A<'a>{type Out;}impl<'a,T:Copy>A<'a>for T{type Out=&'a mut u8;}type P<'a,T>=(T,<T as A<'a>>::Out);fn f<'a>(x:&'a mut u8){let p:P<'_,_>=(0,x);let _:u16=p.0;*p.1=19;*x=73;}fn main(){}",
        true,
    ),
    (
        "reject-later-identity-overlap",
        "trait A<'a>{type Out;}impl<'a,T:Copy>A<'a>for T{type Out=&'a mut u8;}type P<'a,T>=(T,<T as A<'a>>::Out);fn f<'a>(x:&'a mut u8){let p:P<'_,_>=(0,x);let _:u16=p.0;*x=73;*p.1=19;}fn main(){}",
        false,
    ),
    (
        "later-discarded-repeat",
        "trait A<'a>{type Out;}impl<'a,T:Copy>A<'a>for T{type Out=&'a u8;}type P<'a,T>=(T,[<T as A<'a>>::Out;0]);fn f<'a>(x:&'a mut u8){let p:P<'_,_>=(0,[x;0]);*x=73;let _:u16=p.0;let q=p.1;}fn main(){}",
        true,
    ),
    (
        "fallback-dependent-arithmetic",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type P<T>=(T,<T as A>::Out);fn main(){let p:P<_>=(0,7u8);let y=p.1+1;}",
        true,
    ),
    (
        "reject-dependent-arithmetic-result",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type P<T>=(T,<T as A>::Out);fn main(){let p:P<_>=(0,7u8);let y:u16=p.1+1;}",
        false,
    ),
    (
        "known-element-function-pointer",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=fn(u8);}type P<T>=(T,<T as A>::Out);fn g(_:u8){}fn main(){let p:P<_>=(0u16,g);let _:u16=p.0;(p.1)(7);}",
        true,
    ),
    (
        "known-element-shared-reborrow",
        "trait A<'a>{type Out;}impl<'a,T:Copy>A<'a>for T{type Out=&'a u8;}type P<'a,T>=(T,<T as A<'a>>::Out);fn f<'a>(x:&'a mut u8)->u8{let p:P<'_,_>=(0u16,x);let _:u16=p.0;*p.1}fn main(){}",
        true,
    ),
    (
        "known-element-identity-reborrow",
        "trait A<'a>{type Out;}impl<'a,T:Copy>A<'a>for T{type Out=&'a mut u8;}type P<'a,T>=(T,<T as A<'a>>::Out);fn f<'a>(x:&'a mut u8){let p:P<'_,_>=(0u16,x);let _:u16=p.0;*p.1=19;*x=73;}fn main(){}",
        true,
    ),
    (
        "fully-known-identity-reborrow",
        "trait A<'a>{type Out;}impl<'a,T:Copy>A<'a>for T{type Out=&'a mut u8;}type P<'a,T>=(T,<T as A<'a>>::Out);fn f<'a>(x:&'a mut u8){let p:P<'_,u16>=(0,x);let _:u16=p.0;*p.1=19;*x=73;}fn main(){}",
        true,
    ),
    (
        "known-reversed-shared-reborrow",
        "trait A<'a>{type Out;}impl<'a,T:Copy>A<'a>for T{type Out=&'a u8;}type P<'a,T>=(<T as A<'a>>::Out,T);fn f<'a>(x:&'a mut u8)->u8{let p:P<'_,_>=(x,0u16);let _:u16=p.1;*p.0}fn main(){}",
        false,
    ),
    (
        "reversed-shared-no-copy-bound",
        "trait A<'a>{type Out;}impl<'a,T>A<'a>for T{type Out=&'a u8;}type P<'a,T>=(<T as A<'a>>::Out,T);fn f<'a>(x:&'a mut u8)->u8{let p:P<'_,_>=(x,0u16);let _:u16=p.1;*p.0}fn main(){}",
        false,
    ),
    (
        "reversed-shared-known-input",
        "trait A<'a>{type Out;}impl<'a,T:Copy>A<'a>for T{type Out=&'a u8;}type P<'a,T>=(<T as A<'a>>::Out,T);fn f<'a>(x:&'a mut u8)->u8{let p:P<'_,u16>=(x,0u16);let _:u16=p.1;*p.0}fn main(){}",
        true,
    ),
    (
        "reversed-function-pointer",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=fn(u8);}type P<T>=(<T as A>::Out,T);fn g(_:u8){}fn main(){let p:P<_>=(g,0u16);let _:u16=p.1;(p.0)(7);}",
        false,
    ),
    (
        "reversed-function-no-copy-bound",
        "trait A{type Out;}impl<T>A for T{type Out=fn(u8);}type P<T>=(<T as A>::Out,T);fn g(_:u8){}fn main(){let p:P<_>=(g,0u16);let _:u16=p.1;(p.0)(7);}",
        false,
    ),
    (
        "reversed-function-known-input",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=fn(u8);}type P<T>=(<T as A>::Out,T);fn g(_:u8){}fn main(){let p:P<u16>=(g,0u16);let _:u16=p.1;(p.0)(7);}",
        true,
    ),
    (
        "reversed-explicit-shared-borrow",
        "trait A<'a>{type Out;}impl<'a,T>A<'a>for T{type Out=&'a u8;}type P<'a,T>=(<T as A<'a>>::Out,T);fn f<'a>(x:&'a mut u8)->u8{let p:P<'_,_>=(&*x,0u16);let _:u16=p.1;*p.0}fn main(){}",
        true,
    ),
    (
        "reversed-explicit-function-pointer",
        "trait A{type Out;}impl<T>A for T{type Out=fn(u8);}type P<T>=(<T as A>::Out,T);fn g(_:u8){}fn main(){let p:P<_>=(g as fn(u8),0u16);let _:u16=p.1;(p.0)(7);}",
        true,
    ),
    (
        "fallback-input-no-copy-bound",
        "trait A{type Out;}impl<T>A for T{type Out=u8;}type P<T>=(T,<T as A>::Out);fn main(){let p:P<_>=(0,7u8);let a=p.1;let _:u8=a;}",
        true,
    ),
    (
        "fallback-arithmetic-no-copy-bound",
        "trait A{type Out;}impl<T>A for T{type Out=u8;}type P<T>=(T,<T as A>::Out);fn main(){let p:P<_>=(0,7u8);let y=p.1+1;}",
        true,
    ),
    (
        "ordinary-shared-tuple-second-reborrow",
        "type P<T>=(T,T);fn f(x:&mut u8,y:&mut u8){let p:P<_>=(x,y);*p.1=19;*y=73;}fn main(){}",
        true,
    ),
    (
        "ordinary-shared-tuple-first-moves",
        "type P<T>=(T,T);fn f(x:&mut u8,y:&mut u8){let p:P<_>=(x,y);*p.1=19;*x=73;}fn main(){}",
        false,
    ),
    (
        "ordinary-independent-tuple-moves",
        "fn f(x:&mut u8,y:&mut u8){let p:(_,_)=(x,y);*p.1=19;*y=73;}fn main(){}",
        false,
    ),
];
#[test]
fn symbolic_results_match_independent_language_outcomes() -> Result<(), Box<dyn std::error::Error>>
{
    check(CASES)
}
#[test]
fn contextual_operations_preserve_source_time_coercions() -> Result<(), Box<dyn std::error::Error>>
{
    check(CONTEXTS)
}
const RELATIONSHIPS: &[(&str, &str, bool)] = &[
    (
        "nested-direct-subject",
        "struct W<T>(T);trait A{type Out;}impl<T:Copy>A for W<T>{type Out=u8;}trait B{type Out;}impl B for u8{type Out=u16;}type P<T>=(T,<<W<T>as A>::Out as B>::Out);fn main(){let p:P<_>=(3,7u16);let a=p.1;let _:u8=p.0;let _:u16=a;}",
        true,
    ),
    (
        "nested-erased-const-remains-required",
        "struct G<const N:usize>;struct W<T>(T);trait A{type Out;}impl<const N:usize>A for G<N>{type Out=u8;}trait B{type Out;}impl B for W<u8>{type Out=u16;}type Q<const N:usize>=<W<<G<N>as A>::Out>as B>::Out;fn main(){let _:Q<_>=7u16;}",
        false,
    ),
    (
        "nested-callable-input",
        "struct W<T>(T);trait A<'a>{type Out;}impl<'a,T:Copy>A<'a>for W<T>{type Out=&'a u8;}trait B{type Out;}impl B for fn(&u8){type Out=u16;}type P<T>=(T,<for<'a>fn(<W<T>as A<'a>>::Out)as B>::Out);fn main(){let p:P<_>=(3,7u16);let a=p.1;let _:u8=p.0;let _:u16=a;}",
        true,
    ),
    (
        "nested-callable-static-rejected",
        "struct W<T>(T);trait A<'a>{type Out;}impl<'a,T:Copy>A<'a>for W<T>{type Out=&'a u8;}trait B{type Out;}impl B for fn(&'static u8){type Out=u16;}type P<T>=(T,<for<'a>fn(<W<T>as A<'a>>::Out)as B>::Out);fn main(){let p:P<_>=(3,7u16);let a=p.1;let _:u8=p.0;let _:u16=a;}",
        false,
    ),
    (
        "nested-projected-input-known",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}struct W<T>(T);trait B{type Out;}impl<T:Copy>B for W<T>{type Out=u16;}type P<T>=(T,<W<<T as A>::Out>as B>::Out);fn main(){let p:P<_>=(3u8,7u16);let _:u16=p.1;}",
        true,
    ),
    (
        "nested-projected-input-numeric",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}struct W<T>(T);trait B{type Out;}impl<T:Copy>B for W<T>{type Out=u16;}type P<T>=(T,<W<<T as A>::Out>as B>::Out);fn main(){let p:P<_>=(3,7u16);let x=p.1;let _:u8=p.0;let _:u16=x;}",
        true,
    ),
    (
        "nested-projected-input-known-concrete",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}struct W<T>(T);trait B{type Out;}impl B for W<u8>{type Out=u16;}type P<T>=(T,<W<<T as A>::Out>as B>::Out);fn main(){let p:P<_>=(3u8,7u16);let _:u16=p.1;}",
        true,
    ),
    (
        "nested-projected-input-numeric-concrete",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}struct W<T>(T);trait B{type Out;}impl B for W<u8>{type Out=u16;}type P<T>=(T,<W<<T as A>::Out>as B>::Out);fn main(){let p:P<_>=(3,7u16);let x=p.1;let _:u8=p.0;let _:u16=x;}",
        true,
    ),
    (
        "symbolic-result-invariant-capture",
        "struct W<T>(T);trait A{type Out;}impl<T:Copy>A for W<T>{type Out=T;}type F<T>=for<'a>fn(&mut <W<T>as A>::Out,&'a u8);fn g<'a>(_:&mut &'a u8,_:&'a u8){}fn main(){let _:F<_>=g;}",
        false,
    ),
    (
        "symbolic-result-output-capture",
        "struct W<T>(T);trait A{type Out;}impl<T:Copy>A for W<T>{type Out=T;}type F<T>=for<'a>fn(&'a u8)-><W<T>as A>::Out;fn g<'a>(x:&'a u8)->&'a u8{x}fn main(){let _:F<_>=g;}",
        false,
    ),
    (
        "later-array-context",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=[u8;2];}type P<T>=(<T as A>::Out,T);fn main(){let _:P<_>=([7u8,9u8],0u16);}",
        true,
    ),
    (
        "later-explicit-borrow-context",
        "trait A<'a>{type Out;}impl<'a,T:Copy>A<'a>for T{type Out=&'a u8;}type P<'a,T>=(<T as A<'a>>::Out,T);fn main(){let _:P<'_,_>=(&7u8,0u16);}",
        true,
    ),
    (
        "later-block-context",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type P<T>=(<T as A>::Out,T);fn main(){let _:P<_>=({7u8},0u16);}",
        true,
    ),
    (
        "symbolic-result-covariant-input",
        "struct W<T>(T);trait A{type Out;}impl<T:Copy>A for W<T>{type Out=T;}type F<T>=for<'a>fn(<W<T>as A>::Out,&'a u8);fn g<'a>(_:&'a u8,_:&'a u8){}fn main(){let _:F<_>=g;}",
        true,
    ),
    (
        "symbolic-result-covariant-tuple-input",
        "struct W<T>(T);trait A{type Out;}impl<T:Copy>A for W<T>{type Out=(T,T);}type F<T>=for<'a>fn(<W<T>as A>::Out,&'a u8);fn g<'a>(_:(&'a u8,&'a u8),_:&'a u8){}fn main(){let _:F<_>=g;}",
        true,
    ),
    (
        "symbolic-empty-nested-binder-matches",
        "struct W<T>(T);trait A<'a>{type Out;}impl<'a,T:Copy>A<'a>for W<T>{type Out=&'a u8;}type F<T>=for<'a>fn(fn(<W<T>as A<'a>>::Out),T);fn g<'a>(_:fn(&'a u8),_:u16){}fn main(){let _:F<_>=g;}",
        true,
    ),
    (
        "shared-const-output",
        "struct W<const N:usize>;trait A{type Out;}impl<const N:usize>A for W<N>{type Out=([u8;N],[u8;N]);}type Q<const N:usize>=<W<N>as A>::Out;fn main(){let _:Q<_>=([7u8,9u8],[11u8,13u8]);}",
        true,
    ),
    (
        "shared-const-conflict",
        "struct W<const N:usize>;trait A{type Out;}impl<const N:usize>A for W<N>{type Out=([u8;N],[u8;N]);}type Q<const N:usize>=<W<N>as A>::Out;fn main(){let _:Q<_>=([7u8,9u8],[11u8]);}",
        false,
    ),
    (
        "different-results-do-not-select-headers",
        "struct W<T>(T);trait A{type Out;}impl A for W<u8>{type Out=u8;}impl A for W<u16>{type Out=u16;}type Q<T>=<W<T>as A>::Out;fn main(){let _:Q<_>=7u8;}",
        false,
    ),
    (
        "numeric-result-cannot-select-header",
        "struct W<T>(T);trait A{type Out;}impl A for W<u8>{type Out=u8;}impl A for W<u16>{type Out=u16;}type P<T>=(W<T>,<W<T>as A>::Out);fn main(){let _:P<_>=(W(0),7u8);}",
        false,
    ),
    (
        "symbolic-callable-output",
        "struct W<T>(T);trait A{type Out;}impl<T:Copy>A for W<T>{type Out=(fn(T)->T,T);}type Q<T>=<W<T>as A>::Out;fn f(x:u8)->u8{x}fn main(){let q:Q<_>=(f,7u8);let _:u8=(q.0)(q.1);}",
        true,
    ),
    (
        "symbolic-ambient-result",
        "struct W<T>(T);trait A<'a>{type Out;}impl<'a,T:Copy>A<'a>for W<T>{type Out=&'a u8;}type F<T>=for<'a>fn(<W<T>as A<'a>>::Out,T);fn g(_: &u8,_:u16){}fn main(){let f:F<_>=g;f(&7,9);}",
        true,
    ),
    (
        "symbolic-ambient-result-static-rejected",
        "struct W<T>(T);trait A<'a>{type Out;}impl<'a,T:Copy>A<'a>for W<T>{type Out=&'a u8;}type F<T>=for<'a>fn(<W<T>as A<'a>>::Out,T);fn g(_: &'static u8,_:u16){}fn main(){let _:F<_>=g;}",
        false,
    ),
    (
        "symbolic-empty-nested-binder",
        "struct W<T>(T);trait A<'a>{type Out;}impl<'a,T:Copy>A<'a>for W<T>{type Out=&'a u8;}type F<T>=for<'a>fn(fn(<W<T>as A<'a>>::Out),T);fn g(_:fn(&u8),_:u16){}fn main(){let _:F<_>=g;}",
        false,
    ),
];
#[test]
fn symbolic_result_relationships_preserve_identity_and_binders()
-> Result<(), Box<dyn std::error::Error>> {
    check(RELATIONSHIPS)
}
fn check(cases: &[(&str, &str, bool)]) -> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, source, accepted) in cases {
        rustc::qualify(&format!("symbolic-result-{name}"), source, accepted)?;
        let result = semantic::analyze(source).and_then(|(mut report, mut facts)| {
            let entry =
                rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
            rustgm::lower::instances::build(&report, &mut facts, entry)
                .map(|_| ())
                .map_err(|e| e.to_string())
        });
        if result.is_ok() != accepted {
            failures.push(format!("{name}: {:?}", result.err()));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn symbolic_results_execute_with_later_input_proofs() -> Result<(), Box<dyn std::error::Error>> {
    native::check_cases(
        "symbolic-projection-results",
        &[
            (
                "nested-concrete-header",
                "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}struct W<T>(T);trait B{type Out;}impl B for W<u8>{type Out=u16;}type P<T>=(T,<W<<T as A>::Out>as B>::Out);fn main(){let p:P<_>=(3,7u16);let a=p.1+11u16;let _:u8=p.0;check(a==18);check(p.0==3);}",
                false,
            ),
            (
                "nested-callable-header",
                "struct W<T>(T);trait A<'a>{type Out;}impl<'a,T:Copy>A<'a>for W<T>{type Out=&'a u8;}trait B{type Out;}impl B for fn(&u8){type Out=u16;}type P<T>=(T,<for<'a>fn(<W<T>as A<'a>>::Out)as B>::Out);fn main(){let p:P<_>=(3,7u16);let a=p.1+11u16;let _:u8=p.0;check(a==18);check(p.0==3);}",
                false,
            ),
            (
                "later-input",
                "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type P<T>=(T,<T as A>::Out);fn main(){let p:P<_>=(3,7u8);let a=p.1+5u8;let _:u16=p.0;check(a==12);check(p.0==3);}",
                false,
            ),
            (
                "identity",
                "struct W<T>(T);trait A{type Out;}impl<T:Copy>A for W<T>{type Out=(T,T);}type Q<T>=<W<T>as A>::Out;fn main(){let p:Q<_>=(7u8,9u8);check(p.0+p.1==16);}",
                false,
            ),
            (
                "const-result",
                "struct W<const N:usize>;trait A{type Out;}impl<const N:usize>A for W<N>{type Out=([u8;N],[u8;N]);}type Q<const N:usize>=<W<N>as A>::Out;fn main(){let p:Q<_>=([7u8,9u8],[11u8,13u8]);check(p.0[0]+p.1[1]==20);}",
                false,
            ),
            (
                "implicit-callable",
                "trait A{type Out;}impl<T:Copy>A for T{type Out=fn(u8)->u8;}type P<T>=(T,<T as A>::Out);fn g(x:u8)->u8{x+17}fn main(){let p:P<_>=(3,g);let _:u16=p.0;check((p.1)(7)==24);}",
                false,
            ),
            (
                "reborrow",
                "trait A<'a>{type Out;}impl<'a,T:Copy>A<'a>for T{type Out=&'a mut u8;}type P<'a,T>=(T,<T as A<'a>>::Out);fn f(x:&mut u8){let p:P<'_,_>=(3,x);let _:u16=p.0;*p.1=19;check(*p.1==19);*x=73;}fn main(){let mut x=0;f(&mut x);check(x==73);}",
                false,
            ),
            (
                "ambient",
                "struct W<T>(T);trait A<'a>{type Out;}impl<'a,T:Copy>A<'a>for W<T>{type Out=&'a u8;}type F<T>=for<'a>fn(<W<T>as A<'a>>::Out,T);fn g(x:&u8,y:u16){check(*x==7);check(y==9);}fn main(){let f:F<_>=g;f(&7,9);}",
                false,
            ),
            (
                "associated-constant-public-binder",
                "fn identity(x:&u8)->&u8{x}trait Pick<'a>{type Out;const V:Self::Out;}struct Tag;impl<'a>Pick<'a>for Tag{type Out=for<'b>fn(&'b u8)->&'b u8;const V:Self::Out=identity;}fn get<'a,T:Pick<'a>>()->T::Out{let n=T::V;n}fn main(){let f=get::<Tag>();let x=7u8;check(*f(&x)==7);}",
                false,
            ),
        ],
    )
}

const SCOPED_ASSOCIATED: &[(&str, &str, bool)] = &[
    (
        "ordinary-reference",
        "trait Pick<'a>{type Out;const V:Self::Out;}struct Tag;impl<'a>Pick<'a>for Tag{type Out=&'a u8;const V:Self::Out=&7;}fn get<'a,T:Pick<'a>>()->T::Out{let n=T::V;n}fn main(){check(*get::<Tag>()==7);}",
        true,
    ),
    (
        "ordinary-tuple",
        "trait Pick<'a>{type Out;const V:Self::Out;}struct Tag;impl<'a>Pick<'a>for Tag{type Out=(&'a u8,u8);const V:Self::Out=(&7,9);}fn get<'a,T:Pick<'a>>()->T::Out{let n=T::V;n}fn main(){let x=get::<Tag>();check(*x.0==7);check(x.1==9);}",
        true,
    ),
    (
        "ordinary-tuple-after-wide-body",
        concat!(
            "fn wide(){let _r0=&0u8;let _r1=&1u8;let _r2=&2u8;let _r3=&3u8;let _r4=&4u8;let _r5=&5u8;let _r6=&6u8;let _r7=&7u8;let _r8=&8u8;let _r9=&9u8;",
            "let _r10=&10u8;let _r11=&11u8;let _r12=&12u8;let _r13=&13u8;let _r14=&14u8;let _r15=&15u8;let _r16=&16u8;let _r17=&17u8;let _r18=&18u8;let _r19=&19u8;",
            "let _r20=&20u8;let _r21=&21u8;let _r22=&22u8;let _r23=&23u8;let _r24=&24u8;let _r25=&25u8;let _r26=&26u8;let _r27=&27u8;let _r28=&28u8;let _r29=&29u8;",
            "let _r30=&30u8;let _r31=&31u8;let _r32=&32u8;let _r33=&33u8;let _r34=&34u8;let _r35=&35u8;let _r36=&36u8;let _r37=&37u8;let _r38=&38u8;let _r39=&39u8;",
            "}trait Pick<'a>{type Out;const V:Self::Out;}struct Tag;impl<'a>Pick<'a>for Tag{type Out=(&'a u8,u8);const V:Self::Out=(&7,9);",
            "}fn get<'a,T:Pick<'a>>()->T::Out{let n=T::V;n}fn main(){wide();let x=get::<Tag>();check(*x.0==7);check(x.1==9);}",
        ),
        true,
    ),
    (
        "repeated-distinct-owners",
        "trait Pick<'a>{type Out;const V:Self::Out;}struct Tag;impl<'a>Pick<'a>for Tag{type Out=&'a u8;const V:Self::Out=&7;}fn get<'a,T:Pick<'a>,const N:usize>()->(T::Out,T::Out){let a=T::V;let b=T::V;(a,b)}fn main(){let a=get::<Tag,2>();check(*a.0+*a.1==14);let b=get::<Tag,3>();check(*b.0+*b.1==14);}",
        true,
    ),
    (
        "paid-application",
        "trait Pick<'a>{type Out;const V:Self::Out;}struct Tag;impl<'a>Pick<'a>for Tag{type Out=&'a u8;const V:Self::Out=&7;}fn get<'a,'b,T:Pick<'a>>() -> <T as Pick<'a>>::Out where 'a:'b,'b:'a{<T as Pick<'b>>::V}fn main(){check(*get::<Tag>()==7);}",
        true,
    ),
    (
        "unpaid-application",
        "trait Pick<'a>{type Out;const V:Self::Out;}struct Tag;impl<'a>Pick<'a>for Tag{type Out=&'a u8;const V:Self::Out=&7;}fn get<'a,'b,T:Pick<'a>>() -> <T as Pick<'a>>::Out{<T as Pick<'b>>::V}fn main(){}",
        false,
    ),
    (
        "distinct-applications",
        "trait Pick<'a>{type Out;const V:Self::Out;}struct Tag;impl<'a>Pick<'a>for Tag{type Out=&'a u8;const V:Self::Out=&7;}fn get<'a,'b,T:Pick<'a>+Pick<'b>>() -> <T as Pick<'a>>::Out{<T as Pick<'b>>::V}fn main(){}",
        false,
    ),
    (
        "dangling-initializer",
        "trait Pick<'a>{type Out;const V:Self::Out;}struct Tag;impl<'a>Pick<'a>for Tag{type Out=&'a u8;const V:Self::Out={let n=7;&n};}fn get<'a,T:Pick<'a>>()->T::Out{T::V}fn main(){let _=get::<Tag>();}",
        false,
    ),
    (
        "wrong-result",
        "trait Pick<'a>{type Out;const V:Self::Out;}struct Tag;impl<'a>Pick<'a>for Tag{type Out=&'a u8;const V:Self::Out=&7;}fn get<'a,T:Pick<'a,Out=&'a u8>>()->&'a u16{T::V}fn main(){}",
        false,
    ),
    (
        "unpaid-type-bound",
        "trait A<T>{const C:usize;}struct S;impl<T:'static>A<T>for S{const C:usize=3;}fn value<'a>(_:&'a u8)->usize{<S as A<&'a u8>>::C}fn main(){check(<S as A<&'static u8>>::C==3);}",
        false,
    ),
];
#[test]
fn associated_results_retain_original_and_instance_caller_debts()
-> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, source, accepted) in SCOPED_ASSOCIATED {
        let source = format!("{}{source}", native::CHECK);
        rustc::qualify(&format!("scoped-associated-{name}"), &source, accepted)?;
        let result = semantic::resolve(&source).and_then(|mut report| {
            let analysis = rustgm::pipeline::analyze_source(&report).map_err(|e| e.to_string())?;
            let mut facts = analysis.facts;
            let entry =
                rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
            rustgm::lower::instances::build_prepared(&report, &mut facts, entry, analysis.prepared)
                .map(|_| ())
                .map_err(|e| e.to_string())
        });
        if name == "unpaid-type-bound" {
            assert!(
                result.as_ref().err().is_some_and(
                    |message| message.contains("required lifetime does not outlive its target")
                ),
                "{result:?}"
            );
        }
        if result.is_ok() != accepted {
            failures.push(format!("{name}: {:?}", result.err()));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn associated_results_execute_with_exact_repeated_caller_owners()
-> Result<(), Box<dyn std::error::Error>> {
    let cases = SCOPED_ASSOCIATED
        .iter()
        .filter(|case| case.2)
        .map(|&(name, source, _)| (name, source, false))
        .collect::<Vec<_>>();
    native::check_cases("parallel/scoped-associated-results", &cases)
}
