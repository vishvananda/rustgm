#![no_implicit_prelude]
fn check(b:bool){if !b{let z=0;let _=1/z;}}fn main(){check(usize::MAX==18446744073709551615usize);}
