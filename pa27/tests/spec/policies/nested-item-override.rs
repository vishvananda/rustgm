#[deny(unsafe_op_in_unsafe_fn)]fn f(){#[allow(unsafe_op_in_unsafe_fn)]unsafe fn g(p:*const u8)->u8{*p}}fn main(){}
