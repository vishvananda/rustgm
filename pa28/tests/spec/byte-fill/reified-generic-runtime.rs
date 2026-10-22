#![allow(dead_code,unused_mut)]
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const F:unsafe fn(*mut u32,u8,usize)=std::ptr::write_bytes::<u32>;unsafe fn generic<T>(p:*mut T){unsafe{core::ptr::write_bytes(p,3,1);}}fn main(){let mut x=[0u32;2];unsafe{generic(&raw mut x);F(&raw mut x[1],4,1);}check(x[0]==0x03030303);check(x[1]==0x04040404);}
