#![feature(no_core)]
#![no_core]
extern crate base;
fn unpin<T: base::Unpin>() {}
fn unsafe_unpin<T: base::UnsafeUnpin>() {}
fn freeze<T: base::Freeze>() {}
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { unpin::<base::Pair<u8>>(); unsafe_unpin::<base::Raw>(); freeze::<base::Pair<&u8>>(); 42 }
