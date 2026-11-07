#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 {
    let o = base::Opt::Some(5u8);
    match base::Opt::<u8>::is_some(&o) { true => match base::Opt::unwrap_or(o, 1u8) { 5 => 42, _ => 8 }, false => 7 }
}
