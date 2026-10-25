#![feature(no_core,lang_items,rustc_attrs,decl_macro)]
#![no_core]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[unsafe(rustc_builtin_macro(autodiff_forward))]
pub macro actual($item:item){}
#[unsafe(no_mangle)]pub extern "C" fn answer()->u32{7u32}
