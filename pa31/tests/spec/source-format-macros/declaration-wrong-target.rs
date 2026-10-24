#![feature(no_core,lang_items,rustc_attrs,allow_internal_unsafe,allow_internal_unstable)]
#![no_core]
#[lang="pointee_sized"] pub trait PointeeSized {}
#[lang="meta_sized"] pub trait MetaSized:PointeeSized {}
#[lang="sized"] pub trait Sized:MetaSized {}
#[rustc_builtin_macro(format_args)]
fn marked(){}
#[unsafe(no_mangle)] pub extern "C" fn answer()->u32 { 7u32 }
