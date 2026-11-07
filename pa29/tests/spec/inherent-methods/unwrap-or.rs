#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 {
    let o = base::Opt::Some(40u8);
    let n: base::Opt<u8> = base::Opt::None;
    match o.unwrap_or(0) { 40 => n.unwrap_or(42u8) as i32, _ => 7 }
}
