#![feature(no_core)]
#![no_core]
extern crate base;
fn peek(o: &base::Opt<u8>) -> u8 { match o { base::Opt::Some(v) => *v, base::Opt::None => 0 } }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 {
    let o = base::Opt::Some(42u8);
    match &o { base::Opt::Some(v) => match peek(&o) { 42 => *v as i32, _ => 8 }, base::Opt::None => 7 }
}
