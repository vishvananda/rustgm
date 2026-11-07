#![feature(no_core)]
#![no_core]
extern crate base;
fn double(x: u8) -> u8 { match x { 21 => 42, _ => 0 } }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 {
    let some = base::or(base::map(base::wrap(21u8), double), 0u8);
    let other = base::or(base::map(base::wrap(5u8), |x: u8| match x { 5 => 9, _ => 0 }), 0u8);
    match some { 42 => match other { 9 => 42, _ => 8 }, _ => 7 }
}
