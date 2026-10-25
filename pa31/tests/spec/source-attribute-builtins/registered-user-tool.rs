#![feature(register_tool)]
#![register_tool(custom)]
#![feature(no_core,lang_items,rustc_attrs,decl_macro)]
#![no_core]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[custom::attribute]fn target(){}
#[unsafe(no_mangle)]pub extern "C" fn answer()->u32{7u32}
