#![feature(no_core, lang_items)]
#![no_core]
#![allow(internal_features, dead_code)]
#[lang = "pointee_sized"]
pub trait PointeeSized {}
#[lang = "meta_sized"]
pub trait MetaSized: PointeeSized {}
#[lang = "sized"]
pub trait Sized: MetaSized {}
pub trait Tr { type Out: ?Sized; }
impl<X: ?Sized> Tr for X { type Out = X; }
pub struct P<T: ?Sized + Tr>(<T as Tr>::Out);
pub struct Field<U>(P<P<P<P<P<P<P<P<P<P<P<P<P<P<P<P<P<P<P<P<P<P<P<P<P<P<P<P<P<P<P<U>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>, u8);
