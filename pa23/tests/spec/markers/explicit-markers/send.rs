struct W(*const u8);unsafe impl Send for W{}fn need<T:Send>(){}fn f(){need::<W>();}
fn main(){}
