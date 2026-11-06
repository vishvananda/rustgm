#![feature(new_range)]
#![feature(staged_api,const_trait_impl,auto_traits,negative_impls)]
#![unstable(feature="marker_fixture",issue="none")]
#![feature(no_core,lang_items,rustc_attrs,fundamental)]
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

#[lang="freeze"]pub unsafe auto trait Freeze{}
impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}
#[lang="RangeCopy"]pub struct Half<T>{pub end:T,pub start:T}
#[lang="RangeInclusiveCopy"]pub struct Inclusive<T>{pub last:T,pub start:T}
#[lang="RangeFromCopy"]pub struct From<T>{pub start:T}
#[lang="RangeTo"]pub struct To<T>{pub end:T}
#[lang="RangeToInclusiveCopy"]pub struct ToInclusive<T>{pub last:T}
#[lang="RangeFull"]pub struct Full;
#[unsafe(no_mangle)]pub extern "C" fn modern_start()->u8{let r=23u8..47u8;r.start}
#[unsafe(no_mangle)]pub extern "C" fn modern_last()->u8{let r=23u8..=47u8;r.last}
#[unsafe(no_mangle)]pub extern "C" fn from_start()->u8{let r=23u8..;r.start}
#[unsafe(no_mangle)]pub extern "C" fn to_end()->u8{let r=..47u8;r.end}
#[unsafe(no_mangle)]pub extern "C" fn to_last()->u8{let r=..=47u8;r.last}
#[unsafe(no_mangle)]pub extern "C" fn full_value()->u8{let _=..;1}

