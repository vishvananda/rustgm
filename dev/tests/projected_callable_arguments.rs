//! Explicit projected arguments retain normalization through callable and field substitution.
use crate::native_references as native;
use crate::qualify as rustc;
use crate::semantic;
const CASES: &[(&str, &str, bool)] = &[
    (
        "function-argument",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type P<T>=(T,<T as A>::Out);fn pass<U>(f:fn(U))->fn(U){f}fn g(_:(u16,u8)){}fn main(){let f=pass::<P<_>>(g);f((3u16,7u8));}",
        true,
    ),
    (
        "function-argument-wrong-result",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type P<T>=(T,<T as A>::Out);fn pass<U>(f:fn(U))->fn(U){f}fn g(_:(u16,u32)){}fn main(){let f=pass::<P<_>>(g);}",
        false,
    ),
    (
        "function-argument-reversed",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type P<T>=(<T as A>::Out,T);fn pass<U>(f:fn(U))->fn(U){f}fn g(_:(u8,u16)){}fn main(){let f=pass::<P<_>>(g);f((7u8,3u16));}",
        true,
    ),
    (
        "function-argument-erased",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type P<T>=<T as A>::Out;fn pass<U>(f:fn(U))->fn(U){f}fn g(_:u8){}fn main(){let f=pass::<P<_>>(g);}",
        false,
    ),
    (
        "function-argument-nested",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type P<T>=(T,<T as A>::Out);fn pass<U>(f:fn(fn(U)))->fn(fn(U)){f}fn g(_:fn((u16,u8))){}fn value(_:(u16,u8)){}fn main(){let f=pass::<P<_>>(g);f(value);}",
        true,
    ),
    (
        "function-argument-captured",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type P<T>=(T,<T as A>::Out);fn pass<U>(f:fn(U))->fn(U){f}fn take<U>(_: (U,u8)){}fn f<'a>(x:&'a u8){let h=pass::<P<_>>(take::<&'a u8>);h((x,7u8));}fn main(){}",
        true,
    ),
    (
        "function-argument-static",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type P<T>=(T,<T as A>::Out);fn pass<U>(f:fn(U))->fn(U){f}fn take(_: (&'static u8,u8)){}fn f<'a>(x:&'a u8){let h=pass::<P<_>>(take);h((x,7u8));}fn main(){}",
        false,
    ),
    (
        "function-argument-pointer",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type P<T>=(T,<T as A>::Out);fn pass<U>(f:fn(U))->fn(U){f}fn g(_:(u16,u8)){}fn main(){let f=pass::<P<_>>(g as fn((u16,u8)));f((3u16,7u8));}",
        true,
    ),
    (
        "function-nominal-result",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type P<T>=(T,<T as A>::Out);struct W<U>(fn(U));fn make<U>(f:fn(U))->W<U>{W(f)}fn g(_:(u16,u8)){}fn main(){let w=make::<P<_>>(g);(w.0)((3u16,7u8));}",
        true,
    ),
    (
        "function-argument-shared-input",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type P<T>=(T,<T as A>::Out);fn pair<U>(_:fn(U),_:fn(U)){}fn g(_:(u16,u8)){}fn h(_:(u32,u8)){}fn main(){pair::<P<_>>(g,h);}",
        false,
    ),
    (
        "inherent-generic-method",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type P<T>=(T,<T as A>::Out);struct S;impl S{fn pass<U>(f:fn(U))->fn(U){f}}fn g(_:(u16,u8)){}fn main(){let f=S::pass::<P<_>>(g);f((3u16,7u8));}",
        true,
    ),
    (
        "nominal-constructor",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type P<T>=(T,<T as A>::Out);struct W<U>(fn(U));fn g(_:(u16,u8)){}fn main(){let w=W::<P<_>>(g);(w.0)((3u16,7u8));}",
        true,
    ),
    (
        "nominal-field-annotation",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type P<T>=(T,<T as A>::Out);struct W<U>(fn(U));fn g(_:(u16,u8)){}fn main(){let w:W<P<_>>=W(g);(w.0)((3u16,7u8));}",
        true,
    ),
    (
        "function-late-binder",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type P<T>=(T,<T as A>::Out);fn pass<U>(f:for<'b>fn(&'b U))->for<'b>fn(&'b U){f}fn g(_:&(u16,u8)){}fn main(){let f=pass::<P<_>>(g);f(&(3u16,7u8));}",
        true,
    ),
    (
        "function-late-binder-captured",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type P<T>=(T,<T as A>::Out);fn pass<U>(f:for<'b>fn(&'b U))->for<'b>fn(&'b U){f}fn take<U>(_: &(U,u8)){}fn f<'a>(x:&'a u8){let h=pass::<P<_>>(take::<&'a u8>);h(&(x,7u8));}fn main(){}",
        true,
    ),
    (
        "function-late-binder-result",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type P<T>=(T,<T as A>::Out);fn pass<U>(f:for<'b>fn(&'b U)->&'b U)->for<'b>fn(&'b U)->&'b U{f}fn g(x:&(u16,u8))->&(u16,u8){x}fn main(){let f=pass::<P<_>>(g);let x=(3u16,7u8);let y=f(&x);}",
        true,
    ),
    (
        "function-late-binder-static",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type P<T>=(T,<T as A>::Out);fn pass<U>(f:for<'b>fn(&'b U))->for<'b>fn(&'b U){f}fn g(_:&'static (u16,u8)){}fn main(){let f=pass::<P<_>>(g);}",
        false,
    ),
    (
        "function-static-requirement",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type P<T>=(T,<T as A>::Out);fn pass<U:'static>(f:fn(U))->fn(U){f}fn take<U>(_: (U,u8)){}fn f<'a>(x:&'a u8){let h=pass::<P<_>>(take::<&'a u8>);h((x,7u8));}fn main(){}",
        false,
    ),
    (
        "trait-generic-method",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type P<T>=(T,<T as A>::Out);trait C{fn pass<U>(f:fn(U))->fn(U);}struct S;impl C for S{fn pass<U>(f:fn(U))->fn(U){f}}fn g(_:(u16,u8)){}fn main(){let f=<S as C>::pass::<P<_>>(g);f((3u16,7u8));}",
        true,
    ),
    (
        "projected-result-capture",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=T;}type P<T>=(T,<T as A>::Out);fn pass<U>(f:fn(U))->fn(U){f}fn take<U>(_: (U,U)){}fn f<'a>(x:&'a u8){let h=pass::<P<_>>(take::<&'a u8>);h((x,x));}fn main(){}",
        true,
    ),
    (
        "projected-result-static",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=T;}type P<T>=(T,<T as A>::Out);fn pass<U>(f:fn(U))->fn(U){f}fn take(_: (&'static u8,&'static u8)){}fn f<'a>(x:&'a u8){let h=pass::<P<_>>(take);h((x,x));}fn main(){}",
        false,
    ),
    (
        "projected-trait-lifetime",
        "trait A<'a>{type Out;}impl<'a,T:Copy>A<'a>for T{type Out=&'a u8;}type P<'a,T>=(T,<T as A<'a>>::Out);fn pass<U>(f:fn(U))->fn(U){f}fn take(_: (u16,&u8)){}fn f<'a>(x:&'a u8){let h=pass::<P<'a,_>>(take);h((19,x));}fn main(){}",
        true,
    ),
    (
        "projected-trait-lifetime-nested",
        "trait A<'a>{type Out;}impl<'a,T:Copy>A<'a>for T{type Out=&'a u8;}type P<'a,T>=(T,<T as A<'a>>::Out);fn pass<U>(f:for<'b>fn(&'b U))->for<'b>fn(&'b U){f}fn take(_: &(u16,&u8)){}fn f<'a>(x:&'a u8){let h=pass::<P<'a,_>>(take);h(&(19,x));}fn main(){}",
        true,
    ),
    (
        "projected-trait-static",
        "trait A<'a>{type Out;}impl<'a,T:Copy>A<'a>for T{type Out=&'a u8;}type P<'a,T>=(T,<T as A<'a>>::Out);fn pass<U>(f:fn(U))->fn(U){f}fn take(_: (u16,&'static u8)){}fn f<'a>(x:&'a u8){let h=pass::<P<'a,_>>(take);h((19,x));}fn main(){}",
        false,
    ),
    (
        "result-independent-binders",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=T;}type P<T>=(T,<T as A>::Out);fn pass<U>(f:fn(U))->fn(U){f}fn take(_:(&u8,&u8)){}fn f<'a>(x:&'a u8){let h=pass::<P<_>>(take);h((x,x));}fn main(){}",
        true,
    ),
    (
        "result-correlated-binder",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=T;}type P<T>=(T,<T as A>::Out);fn pass<U>(f:fn(U))->fn(U){f}fn take<'b>(_:(&'b u8,&'b u8)){}fn f<'a>(x:&'a u8){let h=pass::<P<_>>(take);h((x,x));}fn main(){}",
        true,
    ),
    (
        "result-independent-main",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=T;}type P<T>=(T,<T as A>::Out);fn pass<U>(f:fn(U))->fn(U){f}fn take(_:(&u8,&u8)){}fn main(){let h=pass::<P<_>>(take);let x=19;h((&x,&x));}",
        true,
    ),
    (
        "result-explicit-input",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=T;}type P<T>=(T,<T as A>::Out);fn pass<U>(f:fn(U))->fn(U){f}fn take(_:(&u8,&u8)){}fn f<'a>(x:&'a u8){let h=pass::<P<&'a u8>>(take);h((x,x));}fn main(){}",
        true,
    ),
    (
        "result-explicit-pointer",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=T;}type P<T>=(T,<T as A>::Out);fn pass<U>(f:fn(U))->fn(U){f}fn take(_:(&u8,&u8)){}fn f<'a>(x:&'a u8){let h=pass::<P<_>>(take as fn((&'a u8,&'a u8)));h((x,x));}fn main(){}",
        true,
    ),
    (
        "result-correlated-return",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=T;}type P<T>=(T,<T as A>::Out);fn pass<U>(f:fn(U)->U)->fn(U)->U{f}fn take<'b>(x:(&'b u8,&'b u8))->(&'b u8,&'b u8){x}fn f<'a>(x:&'a u8)->&'a u8{let h=pass::<P<_>>(take);let y=h((x,x));y.1}fn main(){}",
        true,
    ),
    (
        "result-correlated-escape",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=T;}type P<T>=(T,<T as A>::Out);fn pass<U>(f:fn(U)->U)->fn(U)->U{f}fn take<'b>(x:(&'b u8,&'b u8))->(&'b u8,&'b u8){x}fn f<'a>(x:&'a u8)->&'static u8{let h=pass::<P<_>>(take);let y=h((x,x));y.1}fn main(){}",
        false,
    ),
    (
        "whole-higher-ranked-argument",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=T;}type P<T>=(T,<T as A>::Out);type C=for<'a>fn(&'a u8);fn pass<U>(f:fn(U))->fn(U){f}fn take(_: (C,C)){}fn cb(_: &u8){}fn main(){let h=pass::<P<_>>(take);h((cb,cb));}",
        true,
    ),
    (
        "nested-higher-ranked-input",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=T;}type P<T>=(T,<T as A>::Out);fn pass<U>(f:fn(fn(U)))->fn(fn(U)){f}fn take(_:for<'a>fn((&'a u8,&'a u8))){}fn cb(_:(&u8,&u8)){}fn main(){let h=pass::<P<_>>(take);h(cb);}",
        false,
    ),
    (
        "plain-type-hole-late-input",
        "fn pass<U>(f:fn(U))->fn(U){f}fn take(_: (&u8,&u8)){}fn main(){let h=pass::<(_,_)>(take);let x=19;h((&x,&x));}",
        true,
    ),
];
#[test]
fn projected_arguments_match_rustc() -> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, source, accepted) in CASES {
        rustc::qualify(&format!("projected-callable-{name}"), source, accepted)?;
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
fn projected_arguments_execute_the_selected_functions() -> Result<(), Box<dyn std::error::Error>> {
    native::check_cases(
        "projected-callable-arguments",
        &[
            (
                "function",
                "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type P<T>=(T,<T as A>::Out);fn pass<U>(f:fn(U))->fn(U){f}fn g(x:(u16,u8)){check(x.0==19);check(x.1==73)}fn main(){let f=pass::<P<_>>(g);f((19u16,73u8));}",
                false,
            ),
            (
                "nested",
                "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type P<T>=(T,<T as A>::Out);fn pass<U>(f:fn(fn(U)))->fn(fn(U)){f}fn g(f:fn((u16,u8))){f((19,73))}fn value(x:(u16,u8)){check(x.0==19);check(x.1==73)}fn main(){let f=pass::<P<_>>(g);f(value);}",
                false,
            ),
            (
                "captured",
                "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type P<T>=(T,<T as A>::Out);fn pass<U>(f:fn(U))->fn(U){f}fn take<U>(x:(U,u8)){check(x.1==73)}fn f<'a>(x:&'a u8){let h=pass::<P<_>>(take::<&'a u8>);h((x,73u8));}fn main(){let x=19;f(&x)}",
                false,
            ),
            (
                "nominal-result",
                "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type P<T>=(T,<T as A>::Out);struct W<U>(fn(U));fn make<U>(f:fn(U))->W<U>{W(f)}fn g(x:(u16,u8)){check(x.0==19);check(x.1==73)}fn main(){let w=make::<P<_>>(g);(w.0)((19u16,73u8));}",
                false,
            ),
            (
                "constructor",
                "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type P<T>=(T,<T as A>::Out);struct W<U>(fn(U));fn g(x:(u16,u8)){check(x.0==19);check(x.1==73)}fn main(){let w=W::<P<_>>(g);(w.0)((19u16,73u8));}",
                false,
            ),
            (
                "late-binder-result",
                "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}type P<T>=(T,<T as A>::Out);fn pass<U>(f:for<'b>fn(&'b U)->&'b U)->for<'b>fn(&'b U)->&'b U{f}fn g(x:&(u16,u8))->&(u16,u8){x}fn main(){let f=pass::<P<_>>(g);let x=(19u16,73u8);let y=f(&x);check(y.0==19);check(y.1==73)}",
                false,
            ),
            (
                "projected-result-capture",
                "trait A{type Out;}impl<T:Copy>A for T{type Out=T;}type P<T>=(T,<T as A>::Out);fn pass<U>(f:fn(U))->fn(U){f}fn take(x:(&u8,&u8)){check(*x.0==19);check(*x.1==19)}fn f<'a>(x:&'a u8){let h=pass::<P<_>>(take);h((x,x));}fn main(){let x=19;f(&x)}",
                false,
            ),
        ],
    )
}
