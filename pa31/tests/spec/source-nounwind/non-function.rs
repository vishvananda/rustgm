#![feature(no_core, lang_items, rustc_attrs)]
#![no_core]
#![allow(internal_features, dead_code)]
#[lang = "pointee_sized"] pub trait PointeeSized {}
#[lang = "meta_sized"] pub trait MetaSized: PointeeSized {}
#[lang = "sized"] pub trait Sized: MetaSized {}
#[lang = "legacy_receiver"] pub trait LegacyReceiver: PointeeSized {}
impl<T: PointeeSized> LegacyReceiver for &T {}
impl<T: PointeeSized> LegacyReceiver for &mut T {}
#[lang = "clone"] pub trait Clone: Sized { #[lang = "clone_fn"] fn clone(&self) -> Self; }
#[lang = "copy"] pub trait Copy: Clone {}
impl Copy for bool {}
impl Clone for bool { fn clone(&self) -> Self { *self } }

#[rustc_nounwind]
const fn ordinary(b: bool) -> bool { if b { false } else { true } }
#[rustc_nounwind]
const extern "C" fn ordinary_c(b: bool) -> bool { if b { false } else { true } }
const READY: bool = ordinary(false);
const C_READY: bool = ordinary_c(false);
#[unsafe(no_mangle)]
pub extern "C" fn observed_rust(b: bool) -> bool {
    if b { READY } else { ordinary(true) }
}
#[unsafe(no_mangle)]
pub extern "C" fn observed_c(b: bool) -> bool {
    if b { C_READY } else { ordinary_c(true) }
}
#[unsafe(no_mangle)]
pub extern "C" fn observed_pointer(b: bool) -> bool {
    let f: fn(bool) -> bool = ordinary;
    f(b)
}

#[rustc_nounwind] pub struct NotFunction;
