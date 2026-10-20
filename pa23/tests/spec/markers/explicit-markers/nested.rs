struct W(*const u8);struct Outer(W);unsafe impl Send for W{}fn need<T:Send>(){}fn f(){need::<Outer>();}
fn main(){}
