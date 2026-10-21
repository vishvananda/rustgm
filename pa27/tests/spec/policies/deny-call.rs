unsafe fn g(){}#[deny(unsafe_op_in_unsafe_fn)]unsafe fn f(){g();}fn main(){}
