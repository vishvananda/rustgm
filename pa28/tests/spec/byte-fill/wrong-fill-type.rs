#![allow(dead_code,unused_mut)]
fn main(){let mut x=0u32;unsafe{core::ptr::write_bytes(&raw mut x,256u32,1);}}
