//! Receiver lookup and adjustments agree with independently declared language outcomes.
use super::cases;

#[test]
fn primitive_and_generic_receivers_use_trait_evidence() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "dot-primitive",
            "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self+1}}fn main(){let x=3u8.f();}",
            true,
        ),
        (
            "dot-rigid",
            "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self+1}}fn g<T:A>(x:T)->u8{x.f()}fn main(){let x=g(3u8);}",
            true,
        ),
        (
            "dot-supertrait",
            "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self+1}}trait B:A{}impl B for u8{}fn g<T:B>(x:T)->u8{x.f()}fn main(){let x=g(3u8);}",
            true,
        ),
        (
            "dot-no-bound",
            "trait A{fn f(self)->u8;}fn g<T>(x:T)->u8{x.f()}fn main(){}",
            false,
        ),
        (
            "dot-borrow",
            "trait A{fn f(&self)->u8;}impl A for u8{fn f(&self)->u8{*self}}fn main(){let x=7u8.f();}",
            true,
        ),
        (
            "dot-generic-method",
            "trait A{fn f<T>(self,x:T)->T;}impl A for u8{fn f<T>(self,x:T)->T{x}}fn main(){let x=3u8.f::<u16>(7);}",
            true,
        ),
    ])
}

#[test]
fn receiver_candidate_order_precedes_callable_arguments() -> Result<(), Box<dyn std::error::Error>>
{
    cases(&[
        (
            "dot-inherent-precedes-trait",
            "struct S;impl S{fn f(&self)->u8{1}}trait A{fn f(&self)->u8;}impl A for S{fn f(&self)->u8{2}}fn main(){let x=S.f();}",
            true,
        ),
        (
            "dot-shared-trait-precedes-mutable-inherent",
            "struct S;impl S{fn f(&mut self)->u8{1}}trait A{fn f(&self)->u8;}impl A for S{fn f(&self)->u8{2}}fn main(){let mut s=S;let x=s.f();}",
            true,
        ),
        (
            "dot-two-traits-ambiguous",
            "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self+1}}trait B{fn f(self)->u8;}impl B for u8{fn f(self)->u8{self}}fn main(){let x=3u8.f();}",
            false,
        ),
        (
            "dot-return-does-not-disambiguate",
            "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self+1}}trait B{fn f(self)->u16;}impl B for u8{fn f(self)->u16{7}}fn main(){let x:u8=3u8.f();}",
            false,
        ),
        (
            "dot-arguments-do-not-disambiguate",
            "trait A{fn f(self,x:u8);}trait B{fn f(self,x:u16);}impl A for u8{fn f(self,x:u8){}}impl B for u8{fn f(self,x:u16){}}fn main(){3u8.f(7u8);}",
            false,
        ),
        (
            "dot-static-method-invalid",
            "trait A{fn f()->u8;}impl A for u8{fn f()->u8{7}}fn main(){let x=3u8.f();}",
            false,
        ),
        (
            "dot-method-own-bound-is-required",
            "trait A{fn f<T:Copy>(&self,x:T);}impl A for u8{fn f<T:Copy>(&self,x:T){}}struct N;fn main(){3u8.f(N);}",
            false,
        ),
    ])
}

#[test]
fn imports_and_module_boundaries_control_eligibility() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "dot-trait-not-in-scope",
            "mod m{pub trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self}}}fn main(){let x=7u8.f();}",
            false,
        ),
        (
            "dot-anonymous-import",
            "mod m{pub trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self}}}use m::A as _;fn main(){let x=7u8.f();}",
            true,
        ),
        (
            "dot-aliased-import",
            "mod m{pub trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self}}}use m::A as Alias;fn main(){let x=7u8.f();}",
            true,
        ),
        (
            "dot-block-import-does-not-leak",
            "mod m{pub trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self}}}fn main(){{use m::A as _;let x=7u8.f();}let y=7u8.f();}",
            false,
        ),
        (
            "dot-parent-module-trait-not-imported",
            "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self+1}}mod m{pub fn g()->u8{7u8.f()}}fn main(){let x=m::g();}",
            false,
        ),
        (
            "dot-duplicate-trait-aliases",
            "mod m{pub trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self}}}use m::A as X;use m::A as Y;fn main(){let x=7u8.f();}",
            true,
        ),
    ])
}

#[test]
fn references_and_slices_preserve_receiver_types() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "dot-reference-self",
            "trait A{fn f(self)->u8;}impl<'a>A for &'a u8{fn f(self)->u8{*self}}fn main(){let n=7u8;let x=(&n).f();}",
            true,
        ),
        (
            "dot-reference-self-borrowed",
            "trait A{fn f(&self)->u8;}impl<'a>A for &'a u8{fn f(&self)->u8{**self}}fn main(){let n=7u8;let x=(&n).f();}",
            true,
        ),
        (
            "dot-slice-unsize",
            "trait A{fn f(&self)->u8;}impl A for [u8]{fn f(&self)->u8{self[0]}}fn main(){let a=[7u8,9];let x=a.f();}",
            true,
        ),
        (
            "dot-mutable-receiver-required",
            "trait A{fn f(&mut self)->u8;}impl A for u8{fn f(&mut self)->u8{*self}}fn main(){let n=7u8;let x=n.f();}",
            false,
        ),
    ])
}

#[test]
fn macros_use_call_site_trait_scopes() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "shadowed-trait-name",
            "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{7}}fn main(){struct A;let x=3u8.f();}",
            true,
        ),
        (
            "bound-trait-outside-scope",
            "mod m{pub trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{7}}}fn g<T:m::A>(x:T)->u8{x.f()}fn main(){let x=g(3u8);}",
            true,
        ),
        (
            "macro-call-site-trait",
            "mod m{pub trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{7}}}macro_rules! call{($x:expr)=>{$x.f()}}fn main(){use m::A;let x=call!(3u8);}",
            true,
        ),
        (
            "macro-definition-trait-only",
            "mod m{pub trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{7}}}mod n{use crate::m::A;#[macro_export]macro_rules! call{($x:expr)=>{$x.f()}}}fn main(){let x=call!(3u8);}",
            false,
        ),
        (
            "macro-use-visible-after-expansion",
            "mod m{pub trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{7}}}macro_rules! enable{()=>{use m::A as _;}}fn main(){enable!();let x=3u8.f();}",
            true,
        ),
        (
            "macro-local-trait-visible-after-expansion",
            "macro_rules! enable{()=>{trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{7}}}}fn main(){enable!();let x=3u8.f();}",
            true,
        ),
    ])
}

#[test]
fn bound_priority_requires_the_declared_self_subject() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "shadowed-glob-trait",
            "mod m{pub trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{7}}}use m::*;struct A;fn main(){let x=3u8.f();}",
            false,
        ),
        (
            "ambiguous-glob-different-methods",
            "mod m{pub trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{7}}}mod n{pub trait A{fn g(self)->u8;}impl A for u8{fn g(self)->u8{9}}}use m::*;use n::*;fn main(){let x=3u8.f();let y=3u8.g();}",
            false,
        ),
        (
            "ambiguous-glob-same-method",
            "mod m{pub trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{7}}}mod n{pub trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{9}}}use m::*;use n::*;fn main(){let x=3u8.f();}",
            true,
        ),
        (
            "bound-priority",
            "trait A{fn f(self)->u8;}trait B{fn f(self)->u8;}impl<T>B for T{fn f(self)->u8{9}}impl A for u8{fn f(self)->u8{7}}fn g<T:A>(x:T)->u8{x.f()}fn main(){let x=g(3u8);}",
            true,
        ),
        (
            "two-bound-traits",
            "trait A{fn f(self)->u8;}trait B{fn f(self)->u8;}fn g<T:A+B>(x:T)->u8{x.f()}fn main(){}",
            false,
        ),
        (
            "bound-on-referent-is-not-reference-self-priority",
            "trait A{fn f(self)->u8;}trait B{fn f(self)->u8;}impl<T>A for &T{fn f(self)->u8{7}}impl<T>B for &T{fn f(self)->u8{9}}fn g<T:A>(x:&T)->u8{x.f()}fn main(){}",
            false,
        ),
        (
            "bound-on-referent-shared-receiver-priority",
            "trait A{fn f(&self)->u8;}trait B{fn f(self)->u8;}impl<T>B for &T{fn f(self)->u8{9}}fn g<T:A>(x:&T)->u8{x.f()}fn main(){}",
            true,
        ),
    ])
}

pub(super) const PA18_BOUNDARY: &str = "trait Get{fn get(&self)->u8;}struct S;impl Get for S{fn get(&self)->u8{7}}fn main(){let _=S.get();}\n";

#[test]
fn former_pa18_boundary_accepts_trait_calls() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[("receiver-former-pa18-boundary", PA18_BOUNDARY, true)])
}
