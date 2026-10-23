#![feature(no_core)]
#![no_core]
extern crate base;
struct Token(u8);
#[unsafe(no_mangle)]pub extern "C" fn answer()->u8 {base::identity(Token(7u8)).0}
