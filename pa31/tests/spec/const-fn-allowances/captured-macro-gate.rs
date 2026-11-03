#![feature(no_core,lang_items,staged_api,allow_internal_unstable)]
#![no_core]
#![stable(feature="root",since="1.0.0")]
#![allow(internal_features,dead_code,unused_variables)]
#[stable(feature="root",since="1.0.0")]#[lang="pointee_sized"]pub trait PointeeSized{}
#[stable(feature="root",since="1.0.0")]#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[stable(feature="root",since="1.0.0")]#[lang="sized"]pub trait Sized:MetaSized{}
#[allow_internal_unstable(rustc_attrs)]macro_rules! make {($($t:tt)*)=>{$($t)*}}make!(#[rustc_allow_const_fn_unstable(door)]const fn local()->u8{7});
