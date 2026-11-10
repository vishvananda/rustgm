#![feature(intrinsics,rustc_attrs)]
#![feature(no_core, lang_items)]
#![no_core]
#![allow(internal_features, dead_code)]
#[lang = "pointee_sized"]
pub trait PointeeSized {}
#[lang = "meta_sized"]
pub trait MetaSized: PointeeSized {}
#[lang = "sized"]
pub trait Sized: MetaSized {}
#[lang = "legacy_receiver"] pub trait LegacyReceiver: PointeeSized {}
impl<T: PointeeSized> LegacyReceiver for &T {}
impl<T: PointeeSized> LegacyReceiver for &mut T {}
#[lang = "clone"] pub trait Clone: Sized { #[lang = "clone_fn"] fn clone(&self) -> Self; }
#[lang = "copy"] pub trait Copy: Clone {}
impl Clone for u8 { fn clone(&self) -> Self { *self } }
impl Copy for u8 {}

pub mod intrinsics {
    #[rustc_intrinsic] pub const fn align_of<T>() -> usize;
}
#[repr(align(8))] pub struct Align8<T>(T);
pub trait Storage { type Out; }
impl Storage for u8 { type Out = Align8<u8>; }
pub struct Atomic<T: Storage> { value: T::Out }
pub struct Plain(Align8<u8>);
pub fn closed_projection() {
    let _: [u8; intrinsics::align_of::<Plain>()] = [0; 8];
}
