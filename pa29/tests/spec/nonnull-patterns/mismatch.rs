#![feature(no_core)]
#![no_core]
extern crate base;
enum Opt<T> { None, Some(T) }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let a: Opt<base::NonNull<u8>> = Opt::None; let _b: Opt<base::NonNull<u16>> = a; 7 }
