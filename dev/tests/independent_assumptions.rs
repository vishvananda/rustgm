//! Independent caller assumptions survive unrelated opaque equations without losing debts.
use crate::native_references as native;
use crate::qualify as rustc;
use crate::semantic;
const CASES: &[(&str, &str, bool, bool)] = &[
    (
        "explicit",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<U>{}trait C{type Out;}impl<T:B<u8>>C for T{type Out=u16;}struct S<T:Copy,U:B<<T as A>::Out>+B<u8>>where<U as C>::Out:Copy{t:T,u:U}fn main(){}\n",
        true,
        false,
    ),
    (
        "reordered",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<U>{}trait C{type Out;}impl<T:B<u8>>C for T{type Out=u16;}struct S<T:Copy,U:B<u8>+B<<T as A>::Out>>where<U as C>::Out:Copy{t:T,u:U}fn main(){}\n",
        true,
        false,
    ),
    (
        "missing",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<U>{}trait C{type Out;}impl<T:B<u8>>C for T{type Out=u16;}struct S<T:Copy,U:B<<T as A>::Out>>where<U as C>::Out:Copy{t:T,u:U}fn main(){}\n",
        false,
        false,
    ),
    (
        "opaque-explicit",
        "trait A{type Out;}trait B<U>{}trait C{type Out;}impl<T:B<u8>>C for T{type Out=u16;}struct S<T:A,U:B<<T as A>::Out>+B<u8>>where<U as C>::Out:Copy{t:T,u:U}fn main(){}\n",
        true,
        false,
    ),
    (
        "distinct-normal-form",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u32;}trait B<U>{}trait C{type Out;}impl<T:B<u8>>C for T{type Out=u16;}struct S<T:Copy,U:B<<T as A>::Out>+B<u8>>where<U as C>::Out:Copy{t:T,u:U}fn main(){}\n",
        true,
        false,
    ),
    (
        "wrong-explicit",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<U>{}trait C{type Out;}impl<T:B<u8>>C for T{type Out=u16;}struct S<T:Copy,U:B<<T as A>::Out>+B<u32>>where<U as C>::Out:Copy{t:T,u:U}fn main(){}\n",
        false,
        false,
    ),
    (
        "other-subject",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<U>{}trait C{type Out;}impl<T:B<u8>>C for T{type Out=u16;}struct S<T:Copy+B<u8>,U:B<<T as A>::Out>>where<U as C>::Out:Copy{t:T,u:U}fn main(){}\n",
        false,
        false,
    ),
    (
        "function-explicit",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<U>{}trait C{type Out;}impl<T:B<u8>>C for T{type Out=u16;}fn f<T:Copy,U:B<<T as A>::Out>+B<u8>>()where<U as C>::Out:Copy{}fn main(){}\n",
        true,
        false,
    ),
    (
        "function-opaque",
        "trait A{type Out;}trait B<U>{}fn need<U:B<u8>>(){}fn f<T:A,U:B<<T as A>::Out>+B<u8>>(){need::<U>();}fn main(){}\n",
        true,
        false,
    ),
    (
        "function-qualified",
        "trait A{type Out;}trait B<U>{fn m()->U;}fn f<T:A,U:B<<T as A>::Out>+B<u8>>()->u8{<U as B<u8>>::m()}fn main(){}\n",
        true,
        false,
    ),
    (
        "lifetime-two",
        "trait B<'a,U>{}fn need<'a,U:B<'a,u8>>(_: &'a u8){}fn f<'a,'b,U:B<'a,u8>+B<'b,u8>>(x:&'a u8,y:&'b u8){need::<U>(x);}fn main(){}\n",
        false,
        false,
    ),
    (
        "lifetime-projected",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<'a,U>{}fn need<'a,U:B<'a,u8>>(_: &'a u8){}fn f<'a,'b,T:Copy,U:B<'a,u8>+B<'b,<T as A>::Out>>(x:&'a u8,y:&'b u8){need::<U>(x);}fn main(){}\n",
        false,
        false,
    ),
    (
        "lifetime-opaque",
        "trait A{type Out;}trait B<'a,U>{}fn need<'a,U:B<'a,u8>>(_: &'a u8){}fn f<'a,'b,T:A,U:B<'a,u8>+B<'b,<T as A>::Out>>(x:&'a u8,y:&'b u8){need::<U>(x);}fn main(){}\n",
        true,
        false,
    ),
    (
        "nominal-run",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\ntrait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<U>{}impl B<u8>for u16{}trait C{type Out;}impl<T:B<u8>>C for T{type Out=u16;}struct S<T:Copy,U:B<<T as A>::Out>+B<u8>>where<U as C>::Out:Copy{t:T,u:U}fn main(){let s=S{t:7u8,u:19u16};check(s.t==7);check(s.u==19);}\n",
        true,
        true,
    ),
    (
        "dispatch-run",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\ntrait A{type Out;}impl A for u8{type Out=u32;}trait B<U>{fn m()->U;}impl B<u8>for u16{fn m()->u8{7}}impl B<u32>for u16{fn m()->u32{43}}fn f<T:A,U:B<<T as A>::Out>+B<u8>>()->u8{<U as B<u8>>::m()}fn main(){check(f::<u8,u16>()==7);}\n",
        true,
        true,
    ),
    (
        "different-scope-missing",
        "trait A{type Out;}trait B<U>{}fn need<U:B<u8>>(){}fn f<T:A,U:B<<T as A>::Out>+B<u8>>(){need::<U>();}fn g<T:A,U:B<<T as A>::Out>>(){need::<U>();}fn main(){}\n",
        false,
        false,
    ),
    (
        "unresolved-method-result",
        "trait A{type Out;}trait B<U>{fn m()->U;}fn f<T:A,U:B<<T as A>::Out>+B<u8>>(){let _=U::m();}fn main(){}\n",
        false,
        false,
    ),
    (
        "opaque-lifetime-debt",
        "trait A{type Out;}trait B<'a,U>{}fn need<'a,U:B<'a,u8>>(_: &'a u8){}fn f<'a,'b,T:A,U:B<'a,u8>+B<'b,<T as A>::Out>>(x:&'a u8,y:&'b u8){need::<'b,U>(y);}fn main(){}\n",
        false,
        false,
    ),
    (
        "opaque-lifetime-equal-reviewed",
        "trait A{type Out;}trait B<'a,U>{}fn need<'a,U:B<'a,u8>>(_: &'a u8){}fn f<'a,'b,T:A,U:B<'a,u8>+B<'b,<T as A>::Out>>(x:&'a u8,y:&'b u8)where 'a:'b,'b:'a{need::<'b,U>(y);}fn main(){}\n",
        true,
        false,
    ),
    (
        "return-context",
        "trait A{type Out;}trait B<U>{fn m()->U;}fn f<T:A,U:B<<T as A>::Out>+B<u8>>()->u8{U::m()}fn main(){}\n",
        true,
        false,
    ),
    (
        "return-dispatch-run",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\ntrait A{type Out;}impl A for u8{type Out=u32;}trait B<U>{fn m()->U;}impl B<u8>for u16{fn m()->u8{7}}impl B<u32>for u16{fn m()->u32{43}}fn f<T:A,U:B<<T as A>::Out>+B<u8>>()->u8{U::m()}fn main(){check(f::<u8,u16>()==7);}\n",
        true,
        true,
    ),
    (
        "reordered-return",
        "trait A{type Out;}trait B<V>{fn m()->V;}fn f<T:A,U:B<u8>+B<<T as A>::Out>>()->u8{U::m()}fn main(){}\n",
        true,
        false,
    ),
    (
        "argument-context",
        "trait A{type Out;}trait B<V>{fn m(x:V)->u8;}fn f<T:A,U:B<<T as A>::Out>+B<u8>>(x:u8)->u8{U::m(x)}fn main(){}\n",
        true,
        false,
    ),
    (
        "pointer-context-run",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\ntrait A{type Out;}impl A for u8{type Out=u32;}trait B<V>{fn m()->V;}impl B<u8>for u16{fn m()->u8{7}}impl B<u32>for u16{fn m()->u32{43}}fn f<T:A,U:B<<T as A>::Out>+B<u8>>()->u8{let m:fn()->u8=U::m;m()}fn main(){check(f::<u8,u16>()==7);}\n",
        true,
        true,
    ),
    (
        "dot-context-run",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\ntrait A{type Out;}impl A for u8{type Out=u32;}trait B<V>{fn m(&self)->V;}impl B<u8>for u16{fn m(&self)->u8{7}}impl B<u32>for u16{fn m(&self)->u32{43}}fn f<T:A,U:B<<T as A>::Out>+B<u8>>(x:&U)->u8{x.m()}fn main(){check(f::<u8,u16>(&3)==7);}\n",
        true,
        true,
    ),
    (
        "wrong-return",
        "trait A{type Out;}trait B<V>{fn m()->V;}fn f<T:A,U:B<<T as A>::Out>+B<u8>>()->u16{U::m()}fn main(){}\n",
        false,
        false,
    ),
    (
        "other-subject",
        "trait A{type Out;}trait B<V>{fn m()->V;}fn f<T:A,U:B<<T as A>::Out>+B<u8>>()->u8{T::m()}fn main(){}\n",
        false,
        false,
    ),
    (
        "independent-caller",
        "trait A{type Out;}trait B<V>{fn m()->V;}fn f<T:A,U:B<<T as A>::Out>+B<u8>>()->u8{U::m()}fn g<T:A,U:B<<T as A>::Out>+B<u16>>()->u8{U::m()}fn main(){}\n",
        false,
        false,
    ),
    (
        "method-priority",
        "trait A{type Out;}trait B<V>{fn m()->V;}trait C{fn m()->u8;}fn f<T:A,U:B<<T as A>::Out>+B<u8>+C>()->u8{U::m()}fn main(){}\n",
        false,
        false,
    ),
    (
        "argument-cannot-choose-trait",
        "trait A{type Out;}trait B<V>{fn m(x:V)->u8;}trait C{fn m(x:u16)->u8;}fn f<T:A,U:B<<T as A>::Out>+B<u8>+C>()->u8{U::m(7u8)}fn main(){}\n",
        false,
        false,
    ),
    (
        "mixed-const-run",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\ntrait A{type Out;}impl A for u8{type Out=u32;}trait B<V,const N:usize>{fn m()->V;}impl B<u8,3>for u16{fn m()->u8{7}}impl B<u32,2>for u16{fn m()->u32{43}}fn f<T:A,U:B<<T as A>::Out,2>+B<u8,3>>()->u8{U::m()}fn main(){check(f::<u8,u16>()==7);}\n",
        true,
        true,
    ),
    (
        "shared-arguments",
        "trait A{type Out;}trait B<V,W>{fn m(x:V)->W;}fn f<T:A,U:B<<T as A>::Out,<T as A>::Out>+B<u8,u8>>()->u8{U::m(7u8)}fn main(){}\n",
        true,
        false,
    ),
    (
        "conflicting-arguments",
        "trait A{type Out;}trait B<V,W>{fn m(x:V)->W;}fn f<T:A,U:B<<T as A>::Out,<T as A>::Out>+B<u8,u8>>()->u16{U::m(7u8)}fn main(){}\n",
        false,
        false,
    ),
    (
        "selected-output",
        "trait A{type Out:Copy;}trait B<V>{fn m()->V;}fn f<T:A,U:B<<T as A>::Out>+B<u8>>()-><T as A>::Out{U::m()}fn main(){}\n",
        true,
        false,
    ),
    (
        "lifetime-correct",
        "trait A{type Out;}trait B<'a,V>{fn m(x:&'a u8)->V;}fn f<'a,'b,T:A,U:B<'a,u8>+B<'b,<T as A>::Out>>(x:&'a u8)->u8{U::m(x)}fn main(){}\n",
        true,
        false,
    ),
    (
        "lifetime-missing",
        "trait A{type Out;}trait B<'a,V>{fn m(x:&'a u8)->V;}fn f<'a,'b,T:A,U:B<'a,u8>+B<'b,<T as A>::Out>>(x:&'b u8)->u8{U::m(x)}fn main(){}\n",
        false,
        false,
    ),
    (
        "lifetime-outlives",
        "trait A{type Out;}trait B<'a,V>{fn m(x:&'a u8)->V;}fn f<'a,'b:'a,T:A,U:B<'a,u8>+B<'b,<T as A>::Out>>(x:&'b u8)->u8{U::m(x)}fn main(){}\n",
        true,
        false,
    ),
    (
        "two-caller-results-run",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\ntrait A{type Out;}impl A for u8{type Out=u32;}trait B<V>{fn m()->V;}impl B<u8>for u16{fn m()->u8{7}}impl B<u32>for u16{fn m()->u32{43}}fn f<T:A,U:B<<T as A>::Out>+B<u8>>()->u8{U::m()}fn g<T:A,U:B<<T as A>::Out>+B<u32>>()->u32{U::m()}fn main(){check(f::<u8,u16>()==7);check(g::<u8,u16>()==43);}\n",
        true,
        true,
    ),
];
#[test]
fn independent_assumptions_match_rustc() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, source, accepted, _) in CASES {
        rustc::qualify(&format!("independent-assumption-{name}"), source, accepted)?;
        let result = semantic::analyze(source).and_then(|(mut report, mut facts)| {
            let entry =
                rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
            rustgm::lower::instances::build(&report, &mut facts, entry)
                .map(|_| ())
                .map_err(|e| e.to_string())
        });
        assert_eq!(result.is_ok(), accepted, "{name}: {result:?}");
        let diagnostic = match name {
            "opaque-lifetime-debt" | "lifetime-missing" => {
                Some("required lifetime does not outlive its target")
            }
            "method-priority" | "argument-cannot-choose-trait" => {
                Some("multiple applicable trait methods")
            }
            "unresolved-method-result" => Some(
                "trait obligation requires an unambiguous matching assumption or implementation",
            ),
            _ => None,
        };
        if let Some(diagnostic) = diagnostic {
            assert!(
                result.is_err_and(|error| error.contains(diagnostic)),
                "{name}"
            );
        }
    }
    Ok(())
}
#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn independent_assumptions_execute() -> Result<(), Box<dyn std::error::Error>> {
    let mut cases = Vec::new();
    for &(name, source, _, execute) in CASES {
        if execute {
            let body = source
                .strip_prefix(native::CHECK)
                .ok_or("missing qualified native check prefix")?;
            cases.push((name, body, false));
        }
    }
    native::check_cases("independent-assumptions", &cases)
}
