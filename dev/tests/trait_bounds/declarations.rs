//! Declaration proofs may use headers only while every implementation remains subject to validation.
use super::cases;

#[test]
fn declaration_selection_1() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "declaration-closed-impl",
            "trait A{}impl A for u8{}fn f()where u8:A{}fn main(){f();}",
            true,
        ),
        (
            "declaration-closed-missing",
            "trait A{}fn f()where u8:A{}fn main(){}",
            false,
        ),
        (
            "declaration-wf-cycle",
            "trait A{}struct S where S:A;impl A for S{}fn main(){}",
            true,
        ),
        (
            "declaration-wf-cycle-invalid-member",
            "trait A{fn f();}struct S where S:A;impl A for S{}fn main(){}",
            false,
        ),
        (
            "declaration-wf-cycle-invalid-body",
            "trait A{fn f();}struct S where S:A;impl A for S{fn f(){let x:u8=false;}}fn main(){}",
            false,
        ),
    ])
}

#[test]
fn declaration_selection_2() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "declaration-super-first",
            "trait B{}trait A:B{}impl B for u8{}impl A for u8{}fn g<T:A>(x:T){}fn main(){g(7u8);}",
            true,
        ),
        (
            "declaration-super-last",
            "trait B{}trait A:B{}impl A for u8{}impl B for u8{}fn g<T:A>(x:T){}fn main(){g(7u8);}",
            true,
        ),
        (
            "declaration-super-conditional-cycle",
            "trait B{}trait A:B{}impl A for u8{}impl<T:A>B for T{}fn main(){}",
            true,
        ),
        (
            "declaration-conditional-cycle",
            "trait A{}impl<T:A>A for T{}fn f()where u8:A{}fn main(){}",
            false,
        ),
        (
            "declaration-conformance-blanket",
            "trait A{}impl<T>A for T{}trait B{fn f<T>();}struct S;impl B for S{fn f<U:A>(){}}fn main(){}",
            true,
        ),
    ])
}

#[test]
fn declaration_selection_3() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "declaration-conformance-bad-header",
            "trait A{fn required();}impl<T>A for T{}trait B{fn f<T>();}struct S;impl B for S{fn f<U:A>(){}}fn main(){}",
            false,
        ),
        (
            "declaration-nominal-lifetime",
            "trait A{}impl<'a>A for &'a u8{}struct W<T:A>(T);fn f<'a>(x:W<&'a u8>){}fn main(){}",
            true,
        ),
        (
            "declaration-nominal-conditional-lifetime",
            "trait A<'a>{}impl<'a,T:'a>A<'a>for T{}struct W<'a,T:A<'a>>(&'a (),T);fn f<'a,'b:'a>(x:W<'a,&'b u8>){}fn main(){}",
            true,
        ),
        (
            "declaration-nominal-conditional-lifetime-missing",
            "trait A<'a>{}impl<'a,T:'a>A<'a>for T{}struct W<'a,T:A<'a>>(&'a (),T);fn f<'a,'b>(x:W<'a,&'b u8>){}fn main(){}",
            false,
        ),
        (
            "declaration-callable-outer-variable",
            "trait A<U>{}impl<T>A<T>for fn(T){}struct W<F:A<U>,U>(F,U);fn f<'a>(x:W<fn(&'a u8),&'a u8>){}fn main(){}",
            true,
        ),
    ])
}

#[test]
fn declaration_selection_4() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "declaration-callable-local-capture",
            "trait A<U>{}impl<T>A<T>for fn(T){}struct W<F:A<U>,U>(F,U);fn f(x:W<for<'a>fn(&'a u8),&'static u8>){}fn main(){}",
            false,
        ),
        (
            "declaration-initializer-impl",
            "trait A{}impl A for u8{}struct W<T:A>(T);static X:W<u8>=W(7);fn main(){}",
            true,
        ),
        (
            "declaration-default-impl",
            "trait A{}impl A for u8{}struct W<T:A=u8>(T);fn main(){}",
            true,
        ),
        (
            "declaration-default-lifetime-impl",
            "trait A{}impl<'a>A for &'a u8{}struct W<T:A=&'static u8>(T);fn main(){}",
            true,
        ),
        (
            "declaration-drop-blanket",
            "trait A{}impl<T>A for T{}struct S<T>(T);impl<T:A>Drop for S<T>{fn drop(&mut self){}}fn main(){}",
            true,
        ),
    ])
}

#[test]
fn declaration_selection_extra_1() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "declaration-two-independent-lifetimes",
            "trait A<'a>{}impl<'a,T:'a>A<'a>for T{}struct W<'a,T:A<'a>>(&'a (),T);fn f<'a,'b>(x:W<'a,&'a u8>,y:W<'b,&'b u8>){}fn main(){}",
            true,
        ),
        (
            "declaration-owner-fields-and-default",
            "trait A{}impl<'a>A for &'a u8{}struct W<T:A>(T);struct S<'a,T:A=&'static u8>{x:W<&'a u8>,t:T}fn main(){}",
            true,
        ),
        (
            "declaration-normalized-copy-assumption",
            "trait A{}impl<T:Copy>A for T{}struct W<T:A>(T);fn f<'a,T>(x:W<(T,&'a u8)>)where (T,&'a u8):Copy{}fn main(){}",
            true,
        ),
        (
            "declaration-normalized-copy-missing",
            "trait A{}impl<T:Copy>A for T{}struct W<T:A>(T);fn f<'a,T>(x:W<(T,&'a u8)>){}fn main(){}",
            false,
        ),
        (
            "declaration-nested-callable-outer",
            "trait A<U>{}impl<T>A<T>for fn(fn(T)){}struct W<F:A<U>,U>(F,U);fn f<'a>(x:W<fn(fn(&'a u8)),&'a u8>){}fn main(){}",
            true,
        ),
    ])
}

#[test]
fn declaration_selection_extra_2() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "declaration-callable-local-preserved",
            "trait A<U>{}impl<T>A<T>for for<'b>fn(&'b u8,T){}struct W<F:A<U>,U>(F,U);fn f<'a>(x:W<for<'c>fn(&'c u8,&'a u8),&'a u8>){}fn main(){}",
            true,
        ),
        (
            "declaration-callable-correlation-mismatch",
            "trait A{}impl A for for<'a>fn(&'a u8,&'a u8){}struct W<T:A>(T);fn f(x:W<for<'a,'b>fn(&'a u8,&'b u8)>){}fn main(){}",
            false,
        ),
        (
            "declaration-super-lifetime-given",
            "trait B<'a>{}trait A<'a>:B<'a>{}impl<'a,T:'a>B<'a>for T{}impl<'a,'b:'a>A<'a>for &'b u8{}fn main(){}",
            true,
        ),
        (
            "declaration-super-lifetime-missing",
            "trait B<'a>{}trait A<'a>:B<'a>{}impl<'a,T:'a>B<'a>for T{}impl<'a,'b>A<'a>for &'b u8{}fn main(){}",
            false,
        ),
        (
            "declaration-method-type-outlives-given",
            "trait A<'a>{}impl<'a,T:'a>A<'a>for T{}trait B{fn f<'a,T:'a>(x:&'a T);}struct S;impl B for S{fn f<'b,U:'b+A<'b>>(x:&'b U){}}fn main(){}",
            true,
        ),
    ])
}

#[test]
fn declaration_selection_extra_3() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "declaration-method-type-outlives-missing",
            "trait A<'a>{}impl<'a,T:'a>A<'a>for T{}trait B{fn f<'a,T>();}struct S;impl B for S{fn f<'b,U:A<'b>>() {}}fn main(){}",
            false,
        ),
        (
            "declaration-drop-type-outlives-given",
            "trait A<'a>{}impl<'a,T:'a>A<'a>for T{}struct S<'a,T>(&'a T);impl<'b,U:A<'b>>Drop for S<'b,U>{fn drop(&mut self){}}fn main(){}",
            true,
        ),
        (
            "declaration-drop-type-outlives-missing",
            "trait A<'a>{}impl<'a,T:'a>A<'a>for T{}struct S<'a,T>(&'a (),T);impl<'b,U:A<'b>>Drop for S<'b,U>{fn drop(&mut self){}}fn main(){}",
            false,
        ),
        (
            "declaration-drop-permuted-lifetimes",
            "trait A<'a>{}impl<'a,T:'a>A<'a>for T{}struct S<'a,'b,T,U>(&'a T,&'b U);impl<'x,'y,V:A<'x>,W:A<'y>>Drop for S<'y,'x,W,V>{fn drop(&mut self){}}fn main(){}",
            true,
        ),
        (
            "declaration-drop-unused-lifetime",
            "trait A<'a>{}impl<'a,T:'a>A<'a>for T{}struct S<T>(T);impl<'x,U:A<'x>>Drop for S<U>{fn drop(&mut self){}}fn main(){}",
            true,
        ),
    ])
}

#[test]
fn declaration_selection_extra_4() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "declaration-closed-conditional-copy",
            "trait A{}impl<T:Copy>A for T{}struct S;fn f()where S:A{}fn main(){}",
            false,
        ),
        (
            "declaration-invalid-header-after-use",
            "trait A{fn required();}trait B{fn f<T>();}struct S;impl B for S{fn f<U:A>(){}}impl<T>A for T{}fn main(){}",
            false,
        ),
        (
            "declaration-two-body-lifetimes",
            "trait A{}impl<'a>A for &'a u8{}struct W<T:A>(T);fn f<'a>(x:W<&'a u8>){}fn g<'b>(x:W<&'b u8>){}fn main(){}",
            true,
        ),
    ])
}

const EXECUTIONS: &[(&str, &str, bool)] = &[
    (
        "closed-bound",
        "trait A{}impl A for u8{}fn value()->u8 where u8:A{7}fn main(){check(value()==7);}",
        false,
    ),
    (
        "nominal-borrow",
        "trait A{}impl<'a>A for &'a u8{}struct W<T:A>(T);fn value<'a>(x:W<&'a u8>)->u8{*x.0}fn main(){let x=9u8;check(value(W(&x))==9);}",
        false,
    ),
    (
        "conditional-outlives",
        "trait A<'a>{}impl<'a,T:'a>A<'a>for T{}struct W<'a,T:A<'a>>(&'a (),T);fn value<'a,'b:'a>(x:W<'a,&'b u8>)->u8{*x.1}fn main(){let x=11u8;check(value(W(&(),&x))==11);}",
        false,
    ),
    (
        "blanket-drop",
        "trait A{}impl<T>A for T{}struct S<'a,T>(&'a mut u8,T);impl<'a,T:A>Drop for S<'a,T>{fn drop(&mut self){*self.0+=1;}}fn main(){let mut count=0u8;{let _s=S(&mut count,7u8);}check(count==1);}",
        false,
    ),
];

#[test]
fn declaration_proofs_support_concrete_lowering() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, source, _) in EXECUTIONS {
        cases(&[(name, &format!("{}{source}", super::native::CHECK), true)])?;
    }
    Ok(())
}

#[test]
#[ignore = "requires the qualified CPPGM LowIR-to-native backend"]
fn declaration_proofs_execute_against_rustc() -> Result<(), Box<dyn std::error::Error>> {
    super::native::check_cases("declaration-implementation-selection", EXECUTIONS)
}

#[test]
fn function_items_keep_declared_obligations_after_signature_validation()
-> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "declared-function-item-given",
            "trait A{}impl A for u8{}struct W<T:A>(T);fn value<T:A>(x:W<T>){}fn main(){let _f=value::<u8>;}",
            true,
        ),
        (
            "declared-function-item-missing",
            "trait A{}impl A for u8{}struct W<T:A>(T);fn value<T:A>(x:W<T>){}fn main(){let _f=value::<u16>;}",
            false,
        ),
        (
            "declared-function-signature-missing",
            "trait A{}impl A for u8{}struct W<T:A>(T);fn value<T>(x:W<T>){}fn main(){}",
            false,
        ),
        (
            "declared-concrete-function-signature-missing",
            "trait A{}impl A for u8{}struct W<T:A>(T);fn value(x:W<u16>){}fn main(){}",
            false,
        ),
    ])
}

#[test]
fn lifetime_occurrences_defer_default_bounds_until_actual_use()
-> Result<(), Box<dyn std::error::Error>> {
    for (name, declaration) in [
        (
            "static-argument",
            "trait A<'a>{}struct S<T=u8>where T:A<'static>{v:T}",
        ),
        (
            "callable-argument",
            "trait A<F>{}struct S<T=u8>where T:A<for<'a>fn(&'a u8)>{v:T}",
        ),
        (
            "reference-argument",
            "trait A<F>{}struct S<T=u8>where T:A<&'static u8>{v:T}",
        ),
        (
            "reference-subject",
            "trait A{}struct S<T:'static=u8>where &'static T:A{v:T}",
        ),
    ] {
        for used in [false, true] {
            let source = format!(
                "{declaration}{}fn main(){{}}",
                if used { "fn force(_:S){}" } else { "" }
            );
            let name = format!("default-{name}-used-{used}");
            cases(&[(&name, &source, !used)])?;
        }
    }
    Ok(())
}
