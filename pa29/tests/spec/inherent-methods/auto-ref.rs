#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 {
    let o = base::Opt::Some(5u8);
    let n: base::Opt<u8> = base::Opt::None;
    let r = &&n;
    match o.is_some() { true => match r.is_some() { false => 42, true => 8 }, false => 7 }
}
