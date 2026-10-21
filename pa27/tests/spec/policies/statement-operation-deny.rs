unsafe fn f(p:*const u8){#[deny(unsafe_op_in_unsafe_fn)]let _=*p;}fn main(){}
