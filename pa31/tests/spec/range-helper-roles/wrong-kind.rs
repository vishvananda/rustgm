#![feature(no_core,lang_items,rustc_attrs,const_trait_impl,staged_api)]
#![no_core]#![allow(internal_features,dead_code)]
#![unstable(feature="range_role_fixture",issue="none")]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="RangeMin"] const MIN:u8=0;
