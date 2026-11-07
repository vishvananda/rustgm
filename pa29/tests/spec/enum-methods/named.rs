#![feature(no_core)]
#![no_core]
extern crate base;
fn pick(o: base::Opt<u8>) -> u8 { match o { base::Opt::Some(v) => v, base::Opt::None => 9 } }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 {
    let none: base::Opt<u8> = base::Opt::None;
    let pair = base::Pair::Two(42u8, 1u8);
    match pick(none) { 9 => match pick(base::map(base::Opt::Some(5u8), |x: u8| x)) { 5 => match pair { base::Pair::Two(a, _) => a as i32, base::Pair::One(_) => 6 }, _ => 8 }, _ => 7 }
}
