#![feature(no_core,lang_items,rustc_attrs,intrinsics)]
#![no_core]
#![allow(internal_features,dead_code)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[rustc_intrinsic]pub const fn overflow_checks()->u8{1}
