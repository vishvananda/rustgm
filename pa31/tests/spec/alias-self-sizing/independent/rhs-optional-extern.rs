#![feature(no_core,lang_items,rustc_attrs,trait_alias,extern_types)]
#![no_core]
#![allow(internal_features,dead_code,unused_variables,unused_imports)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
unsafe extern "C"{type Opaque;}
trait A=?Sized;fn need<T:PointeeSized+A>(){}fn check(){need::<Opaque>();}
