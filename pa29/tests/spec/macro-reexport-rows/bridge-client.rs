#![feature(no_core)]
#![no_core]
extern crate bridge;
pub fn joined() -> u32 {
    let _text: &str = bridge::concat!("a", "b");
    let _answer: u32 = bridge::nested::forty_two!();
    bridge::here!()
}
