#![feature(no_core,rustc_attrs,lang_items)]
#![no_core]
#![allow(internal_features,unused_macros,dead_code)]
#[macro_export]
#[rustc_builtin_macro]
macro_rules! concat { () => {} }
#[rustc_builtin_macro]
macro_rules! stringify { () => {} }
use self::concat as join;
#[rustc_diagnostic_item=join!("alias", stringify!(u8))]pub fn value() {}
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
