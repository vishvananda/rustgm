struct W(*const u8);type Alias=W;unsafe impl Send for Alias{}fn need<T:Send>(){}fn f(){need::<W>();}
fn main(){}
