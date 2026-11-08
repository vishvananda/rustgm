#![feature(no_core,lang_items,auto_traits)]
#![no_core]
extern crate base;
#[lang = "sync"]
pub unsafe auto trait Sync {}
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { 42 }
