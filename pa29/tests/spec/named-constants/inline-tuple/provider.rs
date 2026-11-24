#![feature(no_core, lang_items, rustc_attrs)]
#![no_core]
#![rustc_coherence_is_core]
#![allow(internal_features)]

#[lang = "pointee_sized"] pub trait PointeeSized {}
#[lang = "meta_sized"] pub trait MetaSized: PointeeSized {}
#[lang = "sized"] pub trait Sized: MetaSized {}
#[lang = "legacy_receiver"] pub trait LegacyReceiver: PointeeSized {}
impl<T: PointeeSized> LegacyReceiver for &T {}
#[lang = "clone"] pub trait Clone: Sized { #[lang = "clone_fn"] fn clone(&self) -> Self; }
#[lang = "copy"] pub trait Copy: Clone {}
impl Clone for u8 { fn clone(&self) -> Self { *self } }
impl Copy for u8 {}
impl Clone for bool { fn clone(&self) -> Self { *self } }
impl Copy for bool {}
impl Clone for u16 { fn clone(&self) -> Self { *self } }
impl Copy for u16 {}

// Reduced from core's named Unicode data, forwarding item, and primitive constant.
mod data {
    pub const VERSION: (u8, u8, u8) = (101, 37, 219);
}
pub const VERSION: (u8, u8, u8) = data::VERSION;
impl char {
    pub const INLINE_VERSION: (u8, u8, u8) = VERSION;
}

pub const MIXED: (u8, bool, u16) = (4, true, 0x3d95);
