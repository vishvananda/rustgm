#![feature(no_core)]
#![no_core]
extern crate base;
fn large(v: u8) -> bool { match v { 0..=3 => false, _ => true } }
fn rank(o: base::Opt<u8>) -> u8 {
    match o { base::Opt::Some(v) if large(v) => 42, base::Opt::Some(_) => 8, base::Opt::None => 7 }
}
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { match rank(base::Opt::Some(2u8)) { 8 => rank(base::wrap(4u8)) as i32, _ => 6 } }
