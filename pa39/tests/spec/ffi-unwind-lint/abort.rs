#![feature(no_core, lang_items, rustc_attrs, auto_traits, const_trait_impl, const_destruct)]
#![no_core]
#![allow(internal_features, unused_features)]
#![deny(ffi_unwind_calls, unfulfilled_lint_expectations)]

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

unsafe extern "C-unwind" {
    fn foreign();
}

#[cfg_attr(lint_expected, expect(ffi_unwind_calls))]
pub unsafe extern "C-unwind" fn defined_here() {
    unsafe { foreign() }
}
