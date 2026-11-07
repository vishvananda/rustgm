#![feature(no_core)]
#![no_core]
extern crate base;
fn same(x: u8) -> u8 { x }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 {
    match base::keep(base::Opt::Some(42u8), same) { base::Opt::Some(_) => 42, base::Opt::None => 7 }
}
