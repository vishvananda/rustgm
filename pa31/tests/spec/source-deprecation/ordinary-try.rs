#![feature(lang_items, no_core)]
#![no_core]
#![allow(internal_features, unused_imports, unused_macros)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[macro_export]macro_rules! current{()=>{()}}

#[macro_export]macro_rules! r#try{($value:expr)=>{$value}}
mod prelude{#[expect(deprecated)]pub use crate::{current,r#try};}
