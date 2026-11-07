#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 {
    let o = base::Opt::Some(1u8);
    let n: base::Opt<u8> = base::Opt::None;
    match base::present(&o) { true => match base::present(&n) { false => 42, true => 8 }, false => 7 }
}
