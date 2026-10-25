#![feature(no_core,lang_items,rustc_attrs,decl_macro)]
#![no_core]
#![allow(internal_features,dead_code,unused_macros)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
fn value()->u32{macro choose(){7u32}{#[rustc_builtin_macro(test)]macro choose($item:item){}choose!()}}
#[unsafe(no_mangle)]pub extern "C" fn answer()->u32{value()}
