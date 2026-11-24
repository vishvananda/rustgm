#![feature(no_core, lang_items, rustc_attrs, auto_traits, const_trait_impl, const_destruct)]
#![no_core]
#![allow(internal_features, unused_features)]
#![feature(multiple_supertrait_upcastable)]
#![deny(multiple_supertrait_upcastable, unfulfilled_lint_expectations)]

#[lang = "pointee_sized"]
pub trait PointeeSized {}
#[lang = "meta_sized"]
pub trait MetaSized: PointeeSized {}
#[lang = "sized"]
pub trait Sized: MetaSized {}
#[lang = "legacy_receiver"]
pub trait LegacyReceiver: PointeeSized {}
impl<T: PointeeSized> LegacyReceiver for &T {}
#[lang = "clone"]
pub const trait Clone: Sized {
    #[lang = "clone_fn"]
    fn clone(&self) -> Self;
}
#[lang = "copy"]
pub trait Copy: Clone {}
#[lang = "freeze"]
pub unsafe auto trait Freeze {}
#[lang = "destruct"]
pub const trait Destruct: PointeeSized {}

pub trait First {}
pub trait Second {}

#[cfg_attr(not(lint_denied), expect(multiple_supertrait_upcastable))]
pub trait Both: First + Second {}

pub trait Single: First {}
pub trait OnlySized: First + Second + Sized {}
