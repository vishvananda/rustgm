#![feature(no_core,lang_items,rustc_attrs,fundamental,staged_api,intrinsics,lifetime_contract_fixture)]
#![unstable(feature="lifetime_contract_fixture",issue="none")]
#![no_core]
#![allow(internal_features,dead_code,unused_variables)]
#[lang = "pointee_sized"]
#[diagnostic::on_unimplemented(
    message = "values of type `{Self}` may or may not have a size",
    label = "may or may not have a known size"
)]
#[fundamental]
#[rustc_specialization_trait]
#[rustc_deny_explicit_impl]
#[rustc_coinductive]
pub trait PointeeSized {
    // Empty
}
#[lang = "meta_sized"]
#[diagnostic::on_unimplemented(
    message = "the size for values of type `{Self}` cannot be known",
    label = "doesn't have a known size"
)]
#[fundamental]
#[rustc_specialization_trait]
#[rustc_deny_explicit_impl]
// `MetaSized` being coinductive, despite having supertraits, is okay for the same reasons as
// `Sized` above.
#[rustc_coinductive]
pub trait MetaSized: PointeeSized {
    // Empty
}
#[lang = "sized"]
#[diagnostic::on_unimplemented(
    message = "the size for values of type `{Self}` cannot be known at compilation time",
    label = "doesn't have a size known at compile-time"
)]
#[fundamental] // for Default, for example, which requires that `[T]: !Default` be evaluatable
#[rustc_specialization_trait]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
// `Sized` being coinductive, despite having supertraits, is okay as there are no user-written impls,
// and we know that the supertraits are always implemented if the subtrait is just by looking at
// the builtin impls.
#[rustc_coinductive]
pub trait Sized: MetaSized {
    // Empty.
}
#[lang = "legacy_receiver"]
pub trait LegacyReceiver: PointeeSized {}
impl<T: PointeeSized> LegacyReceiver for &T {}
impl<T: PointeeSized> LegacyReceiver for &mut T {}
#[lang = "clone"]
pub trait Clone: Sized { #[lang = "clone_fn"] fn clone(&self) -> Self; }
#[lang = "copy"]
pub trait Copy: Clone {}
impl Clone for u32 { fn clone(&self) -> Self { *self } }
impl Copy for u32 {}
pub enum Maybe<T> { No, Yes(T) }
const fn measure(link: Link0<'_>) -> usize { match link { Link0 { next: _ } => 3 } }
const fn shrink<'a, 'b: 'a>(link: Link0<'b>) -> usize { let short: Link0<'a> = link; measure(short) }
const fn compute() -> usize { shrink(Link0 { next: Maybe::No }) }
pub struct Holder([u8; compute()]);
pub struct Link0<'a> { next: Maybe<&'a Link1<'a>> }
pub struct Link1<'a> { next: Maybe<&'a Link2<'a>> }
pub struct Link2<'a> { text: &'a str }
fn take(h: Holder) -> [u8; 3] { match h { Holder(a) => a } }
