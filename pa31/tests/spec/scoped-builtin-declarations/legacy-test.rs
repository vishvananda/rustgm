#![feature(no_core,lang_items,rustc_attrs,decl_macro)]
#![no_core]
#![allow(internal_features,dead_code,unused_macros)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[rustc_builtin_macro(test)]macro_rules! choose{($item:item)=>{}}
#[choose]fn removed(){missing!();}
fn value()->u32{7u32}
