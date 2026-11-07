#![feature(no_core)]
#![no_core]
extern crate base;
fn double(x: u8) -> u8 { match x { 20 => 42, _ => 0 } }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 {
    let some = base::or(base::map(base::some(20u8), double), 0u8);
    let none = base::or(base::map(base::none(), |x: u8| x), 9u8);
    match some { 42 => match none { 9 => 42, _ => 8 }, _ => 7 }
}
