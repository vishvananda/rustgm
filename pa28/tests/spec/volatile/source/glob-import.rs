use std::ptr::*;fn f(p:*mut u32)->u32{unsafe{write_volatile(p,7);read_volatile(p)}}fn main(){}
