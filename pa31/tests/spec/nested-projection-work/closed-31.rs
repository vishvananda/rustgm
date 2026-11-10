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
pub trait Tr { type Out: ?Sized; }
impl<X: ?Sized> Tr for X { type Out = X; }
pub struct P<T: ?Sized + Tr>(<T as Tr>::Out);
pub struct Field(P<P<P<P<P<P<P<P<P<P<P<P<P<P<P<P<P<P<P<P<P<P<P<P<P<P<P<P<P<P<P<u8>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>, u8);
