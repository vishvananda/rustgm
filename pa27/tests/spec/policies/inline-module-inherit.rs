#![deny(unsafe_op_in_unsafe_fn)]mod m{pub unsafe fn f(p:*const u8)->u8{*p}}fn main(){}
