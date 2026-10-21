#![no_implicit_prelude]
extern crate core;
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let _=1u8.lt(&2u8);}
