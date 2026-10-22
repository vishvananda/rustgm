#![allow(dead_code,unused_mut)]
use core::sync::atomic::AtomicU32;static S:AtomicU32=AtomicU32::new(7);const X:()=unsafe{core::ptr::write_bytes(&S as *const AtomicU32 as *mut AtomicU32,0,0)};fn main(){let _=X;}
