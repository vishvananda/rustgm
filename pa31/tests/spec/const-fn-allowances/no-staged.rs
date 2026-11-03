#![feature(no_core,lang_items,rustc_attrs)]
#![no_core]

#![allow(internal_features,dead_code,unused_variables)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[rustc_allow_const_fn_unstable(door)]
const fn empty()->u8{7}
