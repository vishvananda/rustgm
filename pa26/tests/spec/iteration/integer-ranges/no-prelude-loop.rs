#![no_implicit_prelude]
fn check(b:bool){if !b{let z=0;let _=1/z;}}fn main(){let mut n=0u8;for x in 1u8..4u8{n+=x;}check(n==6);}
