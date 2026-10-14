extern "\x52ust" fn helper(_: &S){}struct S;impl Drop for S{extern r"Rust" fn drop(&mut self){let f:extern "Rust" fn(&S)=helper;f(self);}}fn main(){}
