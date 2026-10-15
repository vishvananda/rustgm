//! Implicit trait lookup retains typed prefixes, scoped candidates and ordinary callable proof.
use super::cases;

#[test]
fn trait_lookup_uses_call_site_scopes_and_expansion_imports()
-> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "prefix-shadowed-trait-name",
            "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{7}}fn main(){struct A;let x=u8::f(3);}",
            true,
        ),
        (
            "prefix-bound-trait-outside-scope",
            "mod m{pub trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{7}}}fn g<T:m::A>(x:T)->u8{T::f(x)}fn main(){let x=g(3u8);}",
            true,
        ),
        (
            "prefix-macro-call-site-trait",
            "mod m{pub trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{7}}}macro_rules! call{($x:expr)=>{<u8>::f($x)}}fn main(){use m::A;let x=call!(3u8);}",
            true,
        ),
        (
            "prefix-macro-definition-trait-only",
            "mod m{pub trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{7}}}mod n{use crate::m::A;#[macro_export]macro_rules! call{($x:expr)=>{<u8>::f($x)}}}fn main(){let x=call!(3u8);}",
            false,
        ),
        (
            "prefix-macro-use-visible-after-expansion",
            "mod m{pub trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{7}}}macro_rules! enable{()=>{use m::A as _;}}fn main(){enable!();let x=u8::f(3);}",
            true,
        ),
        (
            "prefix-macro-local-trait-visible-after-expansion",
            "macro_rules! enable{()=>{trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{7}}}}fn main(){enable!();let x=u8::f(3);}",
            true,
        ),
        (
            "prefix-parent-trait-not-imported",
            "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{7}}mod m{pub fn g()->u8{u8::f(3)}}fn main(){}",
            false,
        ),
    ])
}

#[test]
fn retained_glob_bindings_and_parameter_bounds_have_priority()
-> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "prefix-shadowed-glob-trait",
            "mod m{pub trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{7}}}use m::*;struct A;fn main(){let x=u8::f(3);}",
            false,
        ),
        (
            "prefix-ambiguous-glob-different-methods",
            "mod m{pub trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{7}}}mod n{pub trait A{fn g(self)->u8;}impl A for u8{fn g(self)->u8{9}}}use m::*;use n::*;fn main(){let x=u8::f(3);let y=u8::g(3);}",
            false,
        ),
        (
            "prefix-ambiguous-glob-same-method",
            "mod m{pub trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{7}}}mod n{pub trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{9}}}use m::*;use n::*;fn main(){let x=u8::f(3);}",
            true,
        ),
        (
            "prefix-bound-priority",
            "trait A{fn f(self)->u8;}trait B{fn f(self)->u8;}impl<T>B for T{fn f(self)->u8{9}}impl A for u8{fn f(self)->u8{7}}fn g<T:A>(x:T)->u8{T::f(x)}fn main(){let x=g(3u8);}",
            true,
        ),
        (
            "prefix-two-bound-traits",
            "trait A{fn f(self)->u8;}trait B{fn f(self)->u8;}fn g<T:A+B>(x:T)->u8{T::f(x)}fn main(){}",
            false,
        ),
    ])
}

#[test]
fn glob_import_order_is_independent_of_declaration_order() -> Result<(), Box<dyn std::error::Error>>
{
    cases(&[
        (
            "prefix-reversed-glob-imports-f",
            "mod m{pub trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{1}}}mod n{pub trait A{fn g(self)->u8;}impl A for u8{fn g(self)->u8{2}}}use n::*;use m::*;fn main(){let x=u8::f(3);}",
            false,
        ),
        (
            "prefix-reversed-glob-imports-g",
            "mod m{pub trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{1}}}mod n{pub trait A{fn g(self)->u8;}impl A for u8{fn g(self)->u8{2}}}use n::*;use m::*;fn main(){let x=u8::g(3);}",
            true,
        ),
        (
            "prefix-reversed-glob-declarations-f",
            "mod n{pub trait A{fn g(self)->u8;}impl A for u8{fn g(self)->u8{2}}}mod m{pub trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{1}}}use m::*;use n::*;fn main(){let x=u8::f(3);}",
            true,
        ),
        (
            "prefix-reversed-glob-declarations-g",
            "mod n{pub trait A{fn g(self)->u8;}impl A for u8{fn g(self)->u8{2}}}mod m{pub trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{1}}}use m::*;use n::*;fn main(){let x=u8::g(3);}",
            false,
        ),
    ])
}

#[test]
fn unrelated_homonyms_do_not_exhaust_repeated_lookup() -> Result<(), Box<dyn std::error::Error>> {
    use std::fmt::Write as _;
    let mut declarations = String::new();
    for index in 0..1024 {
        write!(
            declarations,
            "mod m{index}{{pub trait A{{fn f(self)->u8;}}}}"
        )?;
    }
    declarations.push_str("trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self}}");
    for rigid in [false, true] {
        let mut source = declarations.clone();
        for index in 0..1024 {
            if rigid {
                write!(source, "fn g{index}<T:A>(x:T)->u8{{T::f(x)}}")?;
            } else {
                write!(source, "fn g{index}()->u8{{u8::f(7)}}")?;
            }
        }
        source.push_str("fn main(){}");
        cases(&[("implicit-unimported-homonyms", &source, true)])?;
    }
    Ok(())
}
