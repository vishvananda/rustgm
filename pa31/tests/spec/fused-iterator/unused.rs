#![feature(prelude_import,no_core,lang_items,rustc_attrs,staged_api,const_trait_impl,auto_traits)]
#![no_core]
#![unstable(feature="fused_fixture",issue="none")]
#![allow(internal_features,dead_code,unused_variables)]
pub mod prelude { pub use crate::Sized; }
#[prelude_import] use crate::prelude::*;
#[lang="pointee_sized"] pub trait PointeeSized {}
#[lang="meta_sized"] pub trait MetaSized:PointeeSized {}
#[lang="sized"] pub trait Sized:MetaSized {}
#[lang="legacy_receiver"] pub trait LegacyReceiver:PointeeSized {}
impl<T:PointeeSized> LegacyReceiver for &T {}
impl<T:PointeeSized> LegacyReceiver for &mut T {}
#[lang="clone"] pub trait Clone:Sized { #[lang="clone_fn"] fn clone(&self)->Self; }
#[lang="copy"] pub trait Copy:Clone {}
#[lang="freeze"] pub unsafe auto trait Freeze {}
impl Clone for u8 { fn clone(&self)->Self { *self } } impl Copy for u8 {}
impl Clone for bool { fn clone(&self)->Self { *self } } impl Copy for bool {}
pub enum Option<T> { None, Some(T) }
#[rustc_const_unstable(feature="const_iter",issue="92476")]
#[lang="iterator"] pub const trait Iterator {
    type Item;
    #[lang="next"] fn next(&mut self)->Option<Self::Item>;
}
impl<I:Iterator+?Sized> Iterator for &mut I {
    type Item=I::Item;
    fn next(&mut self)->Option<I::Item> { (**self).next() }
}
pub mod iter { pub trait Step {} }
pub mod num { pub struct NonZero<T>(pub T); }
#[lang="fused_iterator"] pub trait Alternate {}
