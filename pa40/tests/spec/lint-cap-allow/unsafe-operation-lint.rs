#![allow(dead_code,unused_variables)]
#![deny(unsafe_op_in_unsafe_fn)]
unsafe fn touch(p:*const u8)->u8{*p}
fn main(){}
