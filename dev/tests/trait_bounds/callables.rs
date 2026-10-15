//! Callable-local predicates defer until use; outer predicates keep their declaration scope.
use super::cases;

#[test]
fn predicates_keep_their_own_binders_and_do_not_grant_permissions()
-> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "callable-trait-argument-local",
            "trait A<U>{}struct W<T:A<U>,U>(T,U);fn f<T>(p:for<'a>fn(W<T,&'a u8>)){}fn main(){}",
            true,
        ),
        (
            "callable-deferred-not-assumed",
            "trait A<U>{}struct W<T:A<U>,U>(T,U);fn need<T:A<&'static u8>>(){}fn f<T>(p:for<'a>fn(W<T,&'a u8>)){need::<T>();}fn main(){}",
            false,
        ),
        (
            "callable-predicate-own-local",
            "trait A{}struct W<T:A>(T);fn f(x:for<'a>fn(&'a (),W<for<'b>fn(&'b u8)>)){}fn main(){}",
            false,
        ),
        (
            "callable-predicate-enclosing-local",
            "trait A{}struct W<T:A>(T);fn f(x:for<'a>fn(W<fn(&'a u8)>)){}fn main(){}",
            true,
        ),
        (
            "callable-predicate-captured-outer",
            "trait A{}struct W<T:A>(T);impl<'a>A for for<'b>fn(&'b u8,&'a u8){}fn f<'x>(p:fn(W<for<'b>fn(&'b u8,&'x u8)>)){}fn main(){}",
            true,
        ),
    ])
}

const EXECUTIONS: &[(&str, &str, bool)] = &[
    (
        "pointer-bounded-input",
        "trait A{}impl<'a>A for &'a u8{}struct W<T:A>(T);fn value<'a>(x:W<&'a u8>)->u8{*x.0}fn through(p:for<'a>fn(W<&'a u8>)->u8)->u8{let x=23u8;p(W(&x))}fn main(){check(through(value)==23);}",
        false,
    ),
    (
        "pointer-bounded-output",
        "trait A{}impl<'a>A for &'a u8{}struct W<T:A>(T);fn value<'a>(x:&'a u8)->W<&'a u8>{W(x)}fn through(p:for<'a>fn(&'a u8)->W<&'a u8>)->u8{let x=31u8;*p(&x).0}fn main(){check(through(value)==31);}",
        false,
    ),
    (
        "pointer-trait-lifetime-argument",
        "trait A<'a>{}impl<'a>A<'a>for u8{}struct W<'a,T:A<'a>>(&'a (),T);fn value<'a>(x:W<'a,u8>)->u8{x.1}fn through(p:for<'a>fn(W<'a,u8>)->u8)->u8{let x=();p(W(&x,47))}fn main(){check(through(value)==47);}",
        false,
    ),
];

#[test]
fn bounded_pointer_calls_lower() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, source, _) in EXECUTIONS {
        cases(&[(name, &format!("{}{source}", super::native::CHECK), true)])?;
    }
    Ok(())
}

#[test]
#[ignore = "requires the qualified CPPGM LowIR-to-native backend"]
fn bounded_pointer_calls_execute() -> Result<(), Box<dyn std::error::Error>> {
    super::native::check_cases("callable-nominal-requirements", EXECUTIONS)
}

#[test]
fn local_and_closed_requirements() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "callable-nominal-local-blanket",
            "trait A{}struct W<T:A>(T);impl<'a>A for &'a u8{}fn f(x:for<'a>fn(W<&'a u8>)){}fn main(){}",
            true,
        ),
        (
            "callable-nominal-local-static",
            "trait A{}struct W<T:A>(T);impl A for &'static u8{}fn f(x:for<'a>fn(W<&'a u8>)){}fn main(){}",
            true,
        ),
        (
            "callable-nominal-local-missing",
            "trait A{}struct W<T:A>(T);fn f(x:for<'a>fn(W<&'a u8>)){}fn main(){}",
            true,
        ),
        (
            "callable-nominal-closed-missing",
            "trait A{}struct W<T:A>(T);fn f(x:fn(W<u8>)){}fn main(){}",
            false,
        ),
        (
            "callable-nominal-closed-impl",
            "trait A{}struct W<T:A>(T);impl A for u8{}fn f(x:fn(W<u8>)){}fn main(){}",
            true,
        ),
    ])
}

#[test]
fn outer_predicates_are_lifted() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "callable-nominal-independent-missing",
            "trait A{}struct W<'a,T:A>(&'a (),T);fn f<T>(x:for<'a>fn(W<'a,T>)){}fn main(){}",
            false,
        ),
        (
            "callable-nominal-independent-given",
            "trait A{}struct W<'a,T:A>(&'a (),T);fn f<T:A>(x:for<'a>fn(W<'a,T>)){}fn main(){}",
            true,
        ),
        (
            "callable-nominal-outer-given",
            "trait A<'a>{}struct W<'a,T:A<'a>>(&'a (),T);fn f<'b,T:A<'b>>(x:fn(W<'b,T>)){}fn main(){}",
            true,
        ),
        (
            "callable-nominal-outer-missing",
            "trait A<'a>{}struct W<'a,T:A<'a>>(&'a (),T);fn f<'b,T>(x:fn(W<'b,T>)){}fn main(){}",
            false,
        ),
        (
            "callable-nominal-nested-outer-given",
            "trait A<'a>{}struct W<'a,T:A<'a>>(&'a (),T);fn f<'b,T:A<'b>>(x:for<'a>fn(&'a (),fn(W<'b,T>))){}fn main(){}",
            true,
        ),
        (
            "callable-nominal-nested-outer-missing",
            "trait A<'a>{}struct W<'a,T:A<'a>>(&'a (),T);fn f<'b,T>(x:for<'a>fn(&'a (),fn(W<'b,T>))){}fn main(){}",
            false,
        ),
    ])
}

#[test]
fn nested_binders_and_calls() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "callable-nominal-nested-middle-deferred",
            "trait A<'a>{}struct W<'a,T:A<'a>>(&'a (),T);fn f<T>(x:for<'a>fn(&'a (),fn(W<'a,T>))){}fn main(){}",
            true,
        ),
        (
            "callable-nominal-nested-inner-deferred",
            "trait A<'a>{}struct W<'a,T:A<'a>>(&'a (),T);fn f<T>(x:fn(for<'a>fn(W<'a,T>))){}fn main(){}",
            true,
        ),
        (
            "callable-nominal-call-diverges-missing",
            "trait A{}struct W<T:A>(T);fn f(x:for<'a>fn(W<&'a u8>)){x(loop{});}fn main(){}",
            false,
        ),
        (
            "callable-nominal-call-diverges-blanket",
            "trait A{}struct W<T:A>(T);impl<'a>A for &'a u8{}fn f(x:for<'a>fn(W<&'a u8>)){x(loop{});}fn main(){}",
            true,
        ),
        (
            "callable-nominal-call-static",
            "trait A{}struct W<T:A>(T);impl A for &'static u8{}fn f(x:for<'a>fn(W<&'a u8>)){x(W(&7u8));}fn main(){}",
            true,
        ),
    ])
}

#[test]
fn call_results_and_argument_lifetimes() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "callable-nominal-call-local-static-only",
            "trait A{}struct W<T:A>(T);impl A for &'static u8{}fn f(x:for<'a>fn(W<&'a u8>)){let v=7u8;x(W(&v));}fn main(){}",
            false,
        ),
        (
            "callable-nominal-call-local-blanket",
            "trait A{}struct W<T:A>(T);impl<'a>A for &'a u8{}fn f(x:for<'a>fn(W<&'a u8>)){let v=7u8;x(W(&v));}fn main(){}",
            true,
        ),
        (
            "callable-nominal-call-result-missing",
            "trait A{}struct W<T:A>(T);fn f(x:for<'a>fn(&'a u8)->W<&'a u8>){let v=7u8;x(&v);}fn main(){}",
            false,
        ),
        (
            "callable-nominal-call-result-blanket",
            "trait A{}struct W<T:A>(T);impl<'a>A for &'a u8{}fn f(x:for<'a>fn(&'a u8)->W<&'a u8>){let v=7u8;x(&v);}fn main(){}",
            true,
        ),
    ])
}
