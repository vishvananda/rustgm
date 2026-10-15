//! Override conformance is checked even when no concrete method call is demanded.
use super::cases;

#[test]
fn method_contracts_0() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "conformance-unused-late-added",
            "trait A{fn f();}impl A for u8{fn f<'a>(){}}fn main(){}\n",
            true,
        ),
        (
            "conformance-unused-late-removed",
            "trait A{fn f<'a>();}impl A for u8{fn f(){}}fn main(){}\n",
            true,
        ),
        (
            "conformance-late-alpha-reordered",
            "trait A{fn f<'a,'b>(x:&'a u8,y:&'b u8)->&'a u8;}impl A for u8{fn f<'b,'a>(x:&'a u8,y:&'b u8)->&'a u8{x}}fn main(){}\n",
            true,
        ),
        (
            "conformance-early-added",
            "trait A{fn f();}impl A for u8{fn f<'a:'a>(){}}fn main(){}\n",
            false,
        ),
        (
            "conformance-late-to-early",
            "trait A{fn f<'a>(x:&'a u8);}impl A for u8{fn f<'a:'a>(x:&'a u8){}}fn main(){}\n",
            false,
        ),
        (
            "conformance-early-to-late",
            "trait A{fn f<'a:'a>(x:&'a u8);}impl A for u8{fn f<'a>(x:&'a u8){}}fn main(){}\n",
            false,
        ),
        (
            "conformance-early-weakened-kept-early",
            "trait A{fn f<'a,T:'a>();}impl A for u8{fn f<'a:'a,T>(){}}fn main(){}\n",
            true,
        ),
        (
            "conformance-early-reordered",
            "trait A{fn f<'a:'a,'b:'b>(x:&'a u8,y:&'b u8)->&'a u8;}impl A for u8{fn f<'b:'b,'a:'a>(x:&'a u8,y:&'b u8)->&'a u8{x}}fn main(){}\n",
            false,
        ),
        (
            "conformance-early-alpha",
            "trait A{fn f<'a:'a>(x:&'a u8);}impl A for u8{fn f<'z:'z>(x:&'z u8){}}fn main(){}\n",
            true,
        ),
        (
            "conformance-sized-weakened",
            "trait A{fn f<T>();}impl A for u8{fn f<T:?Sized>(){}}fn main(){}\n",
            true,
        ),
        (
            "conformance-sized-strengthened",
            "trait A{fn f<T:?Sized>();}impl A for u8{fn f<T>(){}}fn main(){}\n",
            false,
        ),
        (
            "conformance-extra-type-parameter",
            "trait A{fn f();}impl A for u8{fn f<T>(){}}fn main(){}\n",
            false,
        ),
        (
            "conformance-const-parameter-type",
            "trait A{fn f<const N:u8>();}impl A for u8{fn f<const N:u16>(){}}fn main(){}\n",
            false,
        ),
        (
            "conformance-mut-binding",
            "trait A{fn f(x:u8);}impl A for u8{fn f(mut x:u8){x+=1;}}fn main(){}\n",
            true,
        ),
        (
            "conformance-const-impl-method",
            "trait A{fn f();}impl A for u8{const fn f(){}}fn main(){}\n",
            false,
        ),
    ])
}

#[test]
fn method_contracts_1() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "conformance-unused-missing-required",
            "fn fail(x:u8)->u8{1/x}fn check(ok:bool){if !ok{let _=fail(0);}}trait A{fn f(&self);}struct S;impl A for S{}fn main(){}\n",
            false,
        ),
        (
            "conformance-unused-extra-method",
            "fn fail(x:u8)->u8{1/x}fn check(ok:bool){if !ok{let _=fail(0);}}trait A{}struct S;impl A for S{fn f(&self){}}fn main(){}\n",
            false,
        ),
        (
            "conformance-unused-wrong-receiver",
            "fn fail(x:u8)->u8{1/x}fn check(ok:bool){if !ok{let _=fail(0);}}trait A{fn f(&self);}struct S;impl A for S{fn f(&mut self){}}fn main(){}\n",
            false,
        ),
        (
            "conformance-unused-wrong-result",
            "fn fail(x:u8)->u8{1/x}fn check(ok:bool){if !ok{let _=fail(0);}}trait A{fn f(&self)->u8;}struct S;impl A for S{fn f(&self)->u16{7}}fn main(){}\n",
            false,
        ),
        (
            "conformance-unused-wrong-argument",
            "fn fail(x:u8)->u8{1/x}fn check(ok:bool){if !ok{let _=fail(0);}}trait A{fn f(&self,x:u8);}struct S;impl A for S{fn f(&self,x:u16){}}fn main(){}\n",
            false,
        ),
        (
            "conformance-unused-missing-method-generic",
            "fn fail(x:u8)->u8{1/x}fn check(ok:bool){if !ok{let _=fail(0);}}trait A{fn f<T>(&self,x:T);}struct S;impl A for S{fn f(&self,x:u8){}}fn main(){}\n",
            false,
        ),
        (
            "conformance-unused-stronger-method-bound",
            "fn fail(x:u8)->u8{1/x}fn check(ok:bool){if !ok{let _=fail(0);}}trait A{fn f<T>(&self,x:T);}struct S;impl A for S{fn f<T:Copy>(&self,x:T){}}fn main(){}\n",
            false,
        ),
        (
            "conformance-unused-stronger-method-lifetime",
            "fn fail(x:u8)->u8{1/x}fn check(ok:bool){if !ok{let _=fail(0);}}trait A{fn f<'a>(&self,x:&'a u8);}struct S;impl A for S{fn f<'b:'static>(&self,x:&'b u8){}}fn main(){}\n",
            false,
        ),
        (
            "conformance-unused-wrong-return-lifetime",
            "fn fail(x:u8)->u8{1/x}fn check(ok:bool){if !ok{let _=fail(0);}}trait A{fn f<'a,'b>(&self,x:&'a u8,y:&'b u8)->&'a u8;}struct S;impl A for S{fn f<'a,'b>(&self,x:&'a u8,y:&'b u8)->&'b u8{y}}fn main(){}\n",
            false,
        ),
        (
            "conformance-unused-wrong-const-kind",
            "fn fail(x:u8)->u8{1/x}fn check(ok:bool){if !ok{let _=fail(0);}}trait A{fn f<const N:usize>(&self);}struct S;impl A for S{fn f<T>(&self){}}fn main(){}\n",
            false,
        ),
        (
            "conformance-unused-missing-supertrait",
            "fn fail(x:u8)->u8{1/x}fn check(ok:bool){if !ok{let _=fail(0);}}trait A{}trait B:A{}struct S;impl B for S{}fn main(){}\n",
            false,
        ),
        (
            "conformance-unused-unsafe-impl-for-safe-trait",
            "fn fail(x:u8)->u8{1/x}fn check(ok:bool){if !ok{let _=fail(0);}}trait A{}struct S;unsafe impl A for S{}fn main(){}\n",
            false,
        ),
        (
            "conformance-unused-safe-impl-for-unsafe-trait",
            "fn fail(x:u8)->u8{1/x}fn check(ok:bool){if !ok{let _=fail(0);}}unsafe trait A{}struct S;impl A for S{}fn main(){}\n",
            false,
        ),
        (
            "conformance-unused-invalid-default-body",
            "fn fail(x:u8)->u8{1/x}fn check(ok:bool){if !ok{let _=fail(0);}}trait A{fn f<T>(&self,x:T){let _a=x;let _b=x;}}fn main(){}\n",
            false,
        ),
    ])
}

#[test]
fn method_contracts_2() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "conformance-basic-method",
            "trait A{fn f(&self)->u8;}impl A for u8{fn f(&self)->u8{*self}}fn main(){}",
            true,
        ),
        (
            "conformance-default-method",
            "trait A{fn f(&self)->u8{7}}impl A for u8{}fn main(){}",
            true,
        ),
        (
            "conformance-override-default",
            "trait A{fn f(&self)->u8{7}}impl A for u8{fn f(&self)->u8{9}}fn main(){}",
            true,
        ),
        (
            "conformance-generic-trait",
            "trait A<T>{fn f(x:T)->T;}impl<U>A<U>for u8{fn f(x:U)->U{x}}fn main(){}",
            true,
        ),
        (
            "conformance-generic-method",
            "trait A{fn f<T>(x:T)->T;}impl A for u8{fn f<U>(x:U)->U{x}}fn main(){}",
            true,
        ),
        (
            "conformance-weaker-copy",
            "trait A{fn f<T:Copy>(x:T)->T;}impl A for u8{fn f<U>(x:U)->U{x}}fn main(){}",
            true,
        ),
        (
            "conformance-weaker-body-double-move",
            "trait A{fn f<T:Copy>(x:T)->T;}impl A for u8{fn f<U>(x:U)->U{let y=x;x}}fn main(){}",
            false,
        ),
        (
            "conformance-copy-to-clone",
            "trait A{fn f<T:Copy>(x:T)->T;}impl A for u8{fn f<U:Clone>(x:U)->U{x}}fn main(){}",
            true,
        ),
        (
            "conformance-stronger-result",
            "static N:u8=7;trait A{fn f<'a>(x:&'a u8)->&'a u8;}impl A for u8{fn f<'a>(x:&'a u8)->&'static u8{&N}}fn main(){}",
            true,
        ),
        (
            "conformance-weaker-input",
            "trait A{fn f(x:&'static u8)->u8;}impl A for u8{fn f(x:&u8)->u8{*x}}fn main(){}",
            true,
        ),
        (
            "conformance-lifetime-capture",
            "trait A<'a>{fn f(x:&'a u8)->&'a u8;}struct S<'a>(&'a u8);impl<'b>A<'b>for S<'b>{fn f(x:&'b u8)->&'b u8{x}}fn main(){}",
            true,
        ),
        (
            "conformance-const-method",
            "trait A{fn f<const N:usize>(x:[u8;N])->[u8;N];}impl A for u8{fn f<const M:usize>(x:[u8;M])->[u8;M]{x}}fn main(){}",
            true,
        ),
        (
            "conformance-receiver-required",
            "trait A{fn f(&self);}impl A for u8{fn f(x:&Self){}}fn main(){}",
            false,
        ),
        (
            "conformance-receiver-forbidden",
            "trait A{fn f(x:&Self);}impl A for u8{fn f(&self){}}fn main(){}",
            false,
        ),
        (
            "conformance-extra-public",
            "trait A{fn f();}impl A for u8{pub fn f(){}}fn main(){}",
            false,
        ),
        (
            "conformance-unsafe-required",
            "trait A{unsafe fn f();}impl A for u8{fn f(){}}fn main(){}",
            false,
        ),
        (
            "conformance-unsafe-forbidden",
            "trait A{fn f();}impl A for u8{unsafe fn f(){}}fn main(){}",
            false,
        ),
        (
            "conformance-never-not-result-subtyping",
            "trait A{fn f();}impl A for u8{fn f()->!{loop{}}}fn main(){}",
            false,
        ),
    ])
}

#[test]
fn method_bound_implications() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "conformance-user-bound-weaker",
            "trait B{}trait A{fn f<T:B>();}impl A for u8{fn f<U>(){}}fn main(){}",
            true,
        ),
        (
            "conformance-user-bound-stronger",
            "trait B{}trait A{fn f<T>();}impl A for u8{fn f<U:B>(){}}fn main(){}",
            false,
        ),
        (
            "conformance-user-supertrait",
            "trait C{}trait B:C{}trait A{fn f<T:B>();}impl A for u8{fn f<U:C>(){}}fn main(){}",
            true,
        ),
        (
            "conformance-trait-arguments-equal",
            "trait B<T>{}trait A{fn f<T:B<u8>>();}impl A for u8{fn f<U:B<u8>>(){}}fn main(){}",
            true,
        ),
        (
            "conformance-trait-arguments-differ",
            "trait B<T>{}trait A{fn f<T:B<u8>>();}impl A for u8{fn f<U:B<u16>>(){}}fn main(){}",
            false,
        ),
        (
            "conformance-early-outlives-weaker",
            "trait A{fn f<'a:'b,'b:'b>(x:&'a (),y:&'b ());}impl A for u8{fn f<'x:'x,'y:'y>(x:&'x (),y:&'y ()) {}}fn main(){}",
            true,
        ),
        (
            "conformance-early-outlives-stronger",
            "trait A{fn f<'a:'a,'b:'b>(x:&'a (),y:&'b ());}impl A for u8{fn f<'x:'y,'y:'y>(x:&'x (),y:&'y ()) {}}fn main(){}",
            false,
        ),
        (
            "conformance-owner-implied-bounds",
            "trait A<'a,T>{fn f<'b>(x:&'b T)->&'b T;}struct S<'a,T>(&'a T);impl<'a,T>A<'a,T>for S<'a,T>{fn f<'b>(x:&'b T)->&'b T{x}}fn main(){}",
            true,
        ),
        (
            "conformance-mutability-mismatch",
            "trait A{fn f(x:&mut u8);}impl A for u8{fn f(x:&u8){}}fn main(){}",
            false,
        ),
        (
            "conformance-unsafe-match",
            "trait A{unsafe fn f();}impl A for u8{unsafe fn f(){}}fn main(){}",
            true,
        ),
        (
            "conformance-unsized-default",
            "trait A{fn f()where Self:Sized{}}impl A for str{}fn main(){}",
            true,
        ),
        (
            "conformance-callable-binder-alpha",
            "trait A{fn f(x:for<'a,'b>fn(&'a (),&'b ())->&'a ());}impl A for u8{fn f(x:for<'b,'a>fn(&'a (),&'b ())->&'a ()) {}}fn main(){}",
            true,
        ),
        (
            "conformance-callable-binder-correlation",
            "trait A{fn f(x:for<'a,'b>fn(&'a (),&'b ())->&'a ());}impl A for u8{fn f(x:for<'a,'b>fn(&'a (),&'b ())->&'b ()) {}}fn main(){}",
            false,
        ),
    ])
}

#[test]
fn validated_method_records_distinguish_overrides_and_defaults()
-> Result<(), Box<dyn std::error::Error>> {
    let source = "trait A{fn required(&self)->u8;fn defaulted(&self)->u8{7}}impl A for u8{fn required(&self)->u8{*self}}fn main(){}";
    super::rustc::qualify("conformance-retained-members", source, true)?;
    let (report, facts) = super::semantic::analyze(source)?;
    let implementation = facts
        .generics
        .implementations()
        .next()
        .ok_or("missing impl")?;
    assert_eq!(implementation.methods.len(), 2);
    for &(declaration, override_) in &implementation.methods {
        let name = report
            .package
            .names
            .get(report.resolution.definitions[declaration.0 as usize].name);
        let function = facts
            .functions
            .iter()
            .find(|f| f.definition == declaration)
            .ok_or("missing declaration")?;
        if name == "required" {
            let override_ = override_.ok_or("missing required override")?;
            assert_ne!(override_, declaration);
            assert!(function.body.is_none());
            let method = facts
                .functions
                .iter()
                .find(|f| f.definition == override_)
                .ok_or("missing override")?;
            assert!(method.body.is_some());
            assert_eq!(
                facts.associated.owner(override_),
                facts.associated.owner(implementation.owner)
            );
        } else {
            assert_eq!(name, "defaulted");
            assert!(override_.is_none());
            assert!(function.body.is_some());
        }
    }
    Ok(())
}

#[test]
fn method_trait_equations_require_the_trait_lifetime_contract()
-> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "conformance-trait-equal-lifetimes",
            "trait B<'a>{}trait A{fn f<'a:'b,'b:'a,T:B<'a>>();}impl A for u8{fn f<'x:'y,'y:'x,U:B<'y>>(){}}fn main(){}",
            true,
        ),
        (
            "conformance-trait-one-way-lifetimes",
            "trait B<'a>{}trait A{fn f<'a:'b,'b:'b,T:B<'a>>();}impl A for u8{fn f<'x:'y,'y:'y,U:B<'y>>(){}}fn main(){}",
            false,
        ),
    ])
}
