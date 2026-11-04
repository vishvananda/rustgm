#![feature(no_core,lang_items,rustc_attrs)]
#![no_core]
#![allow(internal_features,unused_macros,dead_code)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[macro_export]#[rustc_builtin_macro]macro_rules! concat {()=>{}}
#[macro_export]#[rustc_builtin_macro]macro_rules! stringify {()=>{}}
#[lang=stringify!(not_a_language_item)]pub fn movement<A,B>()->u8{9}
