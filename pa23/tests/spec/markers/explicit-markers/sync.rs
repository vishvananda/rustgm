struct W(*mut u8);unsafe impl Sync for W{}fn need<T:Sync>(){}fn f(){need::<W>();}
fn main(){}
