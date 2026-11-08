#![feature(no_core)]
#![no_core]
extern crate base;
enum Opt<T> { None, Some(T) }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let o: Opt<base::NonNull<str>> = Opt::None; match o { Opt::None => 42, Opt::Some(_) => 7 } }
