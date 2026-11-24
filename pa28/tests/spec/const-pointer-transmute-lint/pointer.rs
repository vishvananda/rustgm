#![feature(no_core, lang_items, intrinsics, rustc_attrs, auto_traits, const_trait_impl, const_destruct)]
#![no_core]
#![allow(internal_features, unused_features)]
#![deny(unfulfilled_lint_expectations)]

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

#[rustc_intrinsic]
pub const unsafe fn transmute<Src, Dst>(src: Src) -> Dst;

#[cfg(lint_expected)]
#[expect(ptr_to_integer_transmute_in_consts)]
pub const unsafe fn in_const(pointer: *const u8) -> usize {
    unsafe { transmute(pointer) }
}

#[cfg(lint_denied)]
#[deny(ptr_to_integer_transmute_in_consts)]
pub const unsafe fn in_const(pointer: *const u8) -> usize {
    unsafe { transmute(pointer) }
}

#[deny(ptr_to_integer_transmute_in_consts)]
pub unsafe fn runtime(pointer: *const u8) -> usize {
    unsafe { transmute(pointer) }
}
