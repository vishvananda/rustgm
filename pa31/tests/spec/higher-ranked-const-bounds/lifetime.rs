#![feature(no_core,lang_items,const_trait_impl)]
#![no_core]
#![allow(internal_features,dead_code,unused_variables)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
pub const trait Bound<'a>{}
const fn demand<T:for<'a> [const] 'a>(){}
