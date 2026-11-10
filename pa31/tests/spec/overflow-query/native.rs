#![feature(no_core,lang_items,rustc_attrs,intrinsics)]
#![no_core]
#![allow(internal_features,dead_code)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[inline(always)]#[rustc_intrinsic]pub const fn overflow_checks()->bool{false}
const VALUE:bool=overflow_checks();
#[unsafe(no_mangle)]pub extern "C" fn constant()->u8{VALUE as u8}
#[unsafe(no_mangle)]pub extern "C" fn runtime()->u8{overflow_checks() as u8}
#[rustc_inherit_overflow_checks]pub const fn inherited()->bool{overflow_checks()}
#[unsafe(no_mangle)]pub extern "C" fn inherited_runtime()->u8{inherited() as u8}
