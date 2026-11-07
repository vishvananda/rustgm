#![feature(no_core)]
#![no_core]
extern crate base;
fn local<'x>() -> base::Info<'x> { let h = base::Held::new("a", 1); base::Info::new(&h, true) }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { match local().flag { true => 42, false => 7 } }
