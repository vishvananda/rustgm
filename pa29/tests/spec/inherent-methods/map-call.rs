#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 {
    match base::Opt::Some(5u8).map(|x| x + 1) { base::Opt::Some(v) => match v { 6 => 42, _ => 8 }, base::Opt::None => 7 }
}
