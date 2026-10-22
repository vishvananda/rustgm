#![allow(dead_code,unused_mut)]
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let mut x=37u32;unsafe{core::ptr::write_bytes(&raw mut x,255,0);}check(x==37);let mut z=();unsafe{core::ptr::write_bytes(&raw mut z,255,18446744073709551615usize);}let _=z;}
