#![feature(no_core)]
#![no_core]
extern crate base;
fn need<T: base::Send>() {}
fn shared<T: base::Sync>() {}
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { need::<base::Cell<u8>>(); need::<&u8>(); need::<base::Unique<u8>>(); need::<base::Loc<'static>>(); need::<base::Foo<u8>>(); shared::<base::Unique<u8>>(); shared::<base::Loc<'static>>(); 42 }
