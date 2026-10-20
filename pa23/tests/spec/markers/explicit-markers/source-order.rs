fn need<T:Send>(){}fn f(){need::<W>();}unsafe impl Send for W{}struct W(*const u8);
fn main(){}
