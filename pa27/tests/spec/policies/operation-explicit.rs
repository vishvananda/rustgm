#[deny(unsafe_op_in_unsafe_fn)]unsafe fn f(p:*const u8)->u8{unsafe{*p}}fn main(){}
