#![feature(no_core, lang_items, rustc_attrs)]
#![no_core]
#![allow(internal_features)]

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
pub trait Clone: Sized {
    #[lang = "clone_fn"]
    fn clone(&self) -> Self;
}
#[lang = "copy"]
pub trait Copy: Clone {}
impl Clone for u8 {
    fn clone(&self) -> Self {
        *self
    }
}
impl Copy for u8 {}

pub const fn value<const N: u8>() -> u8 {
    const { const { N } }
}

pub fn reborrow<'a, T>(value: &'a T) -> &'a T {
    value
}

pub const fn bytes<const N: usize>() -> [u8; N] {
    [7; N]
}

pub trait Step {
    fn step(self) -> u8;
}

impl Step for u8 {
    fn step(self) -> u8 {
        self
    }
}

pub fn invoke<T: Step>(value: T) -> u8 {
    value.step()
}
