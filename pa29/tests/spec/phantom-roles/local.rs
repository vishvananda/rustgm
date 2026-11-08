#![feature(no_core)]
#![no_core]
extern crate base;
struct Mark<T>(base::PhantomData<T>);
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let _m = Mark::<u16>(base::PhantomData); 42 }
