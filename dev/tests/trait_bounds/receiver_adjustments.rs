//! Receiver lookup and adjustments agree with independently declared language outcomes.
use super::cases;

#[test]
fn mutable_reference_receivers_reborrow_the_original_place()
-> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "reference-self-reborrows-mutable-reference",
            "trait A{fn f(self)->u8;}impl A for &mut u8{fn f(self)->u8{*self}}fn main(){let mut n=7u8;let r=&mut n;let x=r.f();*r=9;}",
            true,
        ),
        (
            "borrowed-receiver-reborrows-mutable-reference",
            "trait A{fn f(&mut self)->u8;}impl A for u8{fn f(&mut self)->u8{*self}}fn main(){let mut n=7u8;let r=&mut n;let x=r.f();*r=9;}",
            true,
        ),
        (
            "reference-self-borrowed-reborrows-outer-reference",
            "trait A{fn f(&mut self)->u8;}impl A for &mut u8{fn f(&mut self)->u8{**self}}fn main(){let mut n=7u8;let mut r=&mut n;let rr=&mut r;let x=rr.f();**rr=9;}",
            true,
        ),
        (
            "explicit-self-reference-value",
            "trait A{fn f(self)->u8;}impl A for &u8{fn f(self)->u8{*self}}fn main(){let n=7u8;let x=n.f();}",
            true,
        ),
    ])
}

#[test]
fn bounded_referents_and_explicit_reference_depths_select_methods()
-> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "hidden-bound-autoref",
            "mod m{pub trait A{fn f(&self)->u8;}impl A for u8{fn f(&self)->u8{*self}}}fn g<T:m::A>(x:T)->u8{x.f()}fn main(){let x=g(7u8);}",
            true,
        ),
        (
            "hidden-bound-reference-receiver",
            "mod m{pub trait A{fn f(&self)->u8;}impl A for u8{fn f(&self)->u8{*self}}}fn g<T:m::A>(x:&T)->u8{x.f()}fn main(){let x=g(&7u8);}",
            true,
        ),
        (
            "bound-shared-priority",
            "trait A{fn f(&self)->u8;}trait B{fn f(self)->u8;}impl<T>B for &T{fn f(self)->u8{9}}impl A for u8{fn f(&self)->u8{7}}fn g<T:A>(x:T)->u8{x.f()}fn main(){let x=g(7u8);}",
            true,
        ),
        (
            "double-reference-receiver",
            "trait A{fn f(self:&&Self)->u8;}impl A for u8{fn f(self:&&Self)->u8{**self}}fn main(){let n=7u8;let x=(&n).f();}",
            true,
        ),
        (
            "double-autoref-is-not-automatic",
            "trait A{fn f(self:&&Self)->u8;}impl A for u8{fn f(self:&&Self)->u8{**self}}fn main(){let n=7u8;let x=n.f();}",
            false,
        ),
    ])
}

#[test]
fn numeric_receiver_domains_retain_inference_and_fallback() -> Result<(), Box<dyn std::error::Error>>
{
    cases(&[
        (
            "numeric-receiver-inferred-from-single-impl",
            "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self}}fn main(){let x=7.f();}",
            true,
        ),
        (
            "numeric-explicit-receiver",
            "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self}}fn main(){let x=7u8.f();}",
            true,
        ),
        (
            "numeric-default-selects-i32",
            "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{7}}impl A for i32{fn f(self)->u8{9}}fn main(){let x=7.f();}",
            true,
        ),
        (
            "numeric-default-without-i32-impl",
            "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{7}}impl A for u16{fn f(self)->u8{9}}fn main(){let x=7.f();}",
            false,
        ),
        (
            "numeric-trait-names-ambiguous",
            "trait A{fn f(self)->u8;}trait B{fn f(self)->u8;}impl A for u8{fn f(self)->u8{7}}impl B for i32{fn f(self)->u8{9}}fn main(){let x=7.f();}",
            false,
        ),
        (
            "float-unique-implementation",
            "trait A{fn f(self)->u8;}impl A for f32{fn f(self)->u8{7}}fn main(){let x=7.0.f();}",
            true,
        ),
        (
            "unknown-whole-receiver-not-guessed",
            "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{7}}fn g<T>()->T{loop{}}fn main(){let x=g().f();}",
            false,
        ),
    ])
}

#[test]
fn array_unsizing_preserves_mutability_and_reference_self() -> Result<(), Box<dyn std::error::Error>>
{
    cases(&[
        (
            "array-unsize-borrow",
            "trait A{fn f(&self)->u8;}impl A for [u8]{fn f(&self)->u8{self[0]}}fn main(){let x=[7u8,9].f();}",
            true,
        ),
        (
            "array-unsize-mutable",
            "trait A{fn f(&mut self)->u8;}impl A for [u8]{fn f(&mut self)->u8{self[0]=9;self[0]}}fn main(){let mut a=[7u8,3];let x=a.f();}",
            true,
        ),
        (
            "array-shared-unsize-no-mutability",
            "trait A{fn f(&mut self)->u8;}impl A for [u8]{fn f(&mut self)->u8{self[0]}}fn main(){let a=[7u8,3];let r=&a;let x=r.f();}",
            false,
        ),
        (
            "slice-reference-self",
            "trait A{fn f(self)->u8;}impl A for &[u8]{fn f(self)->u8{self[0]}}fn main(){let x=[7u8,9].f();}",
            true,
        ),
    ])
}

#[test]
fn selected_calls_still_require_bounds_lifetimes_and_write_permission()
-> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "method-own-bound-does-not-fallback",
            "trait A{fn f<T>(self,x:T);}struct S;impl S{fn f<T:Copy>(self,x:T){}}impl A for S{fn f<T>(self,x:T){}}struct N;fn main(){S.f(N);}",
            false,
        ),
        (
            "lifetime-does-not-choose-later-candidate",
            "trait A{fn f(&'static self)->u8;}trait B{fn f(&mut self)->u8;}struct S;impl A for S{fn f(&'static self)->u8{7}}impl B for S{fn f(&mut self)->u8{9}}fn main(){let mut s=S;let x=s.f();}",
            false,
        ),
        (
            "mutable-lvalue-requirement-after-selection",
            "trait A{fn f(&mut self)->u8;}trait B{fn f(&self)->u8;}struct S;impl A for S{fn f(&mut self)->u8{7}}impl B for &S{fn f(&self)->u8{9}}fn main(){let s=S;let x=s.f();}",
            false,
        ),
        (
            "trait-inherent-alias-fallback",
            "trait A{fn f(self)->u8;}struct S<T>(T);impl<T:Copy>S<T>{fn f(self)->u8{7}}impl<T>A for S<T>{fn f(self)->u8{9}}struct N;fn main(){let x=S(N).f();}",
            true,
        ),
    ])
}

#[test]
fn deep_reference_receivers_have_bounded_candidate_work() -> Result<(), Box<dyn std::error::Error>>
{
    for depth in [64, 96] {
        let source = format!(
            "trait A{{fn f(&self)->u8;}}impl A for u8{{fn f(&self)->u8{{*self}}}}fn g(x:{}u8)->u8{{x.f()}}fn main(){{}}",
            "&".repeat(depth)
        );
        super::rustc::qualify("deep-trait-receiver", &source, true)?;
        let (_, facts) = super::semantic::analyze(&source)?;
        assert_eq!(facts.methods.selection_probes, 1);
        assert!(
            facts.methods.selection_steps < depth * 1024,
            "{} steps",
            facts.methods.selection_steps
        );
        super::prove(&source)?;
    }
    Ok(())
}
