#![allow(dead_code,unused_mut)]
unsafe fn f(p:*mut[u8]){unsafe{core::ptr::write_bytes::<[u8]>(p,0,1);}}fn main(){}
