#![feature(no_core,lang_items,rustc_attrs,decl_macro)]
#![no_core]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[rustc_builtin_macro(autodiff_reverse)]
pub macro actual($item:item){ /* compiler built-in */ }
#[unsafe(no_mangle)]pub extern "C" fn answer()->u32{7u32}
