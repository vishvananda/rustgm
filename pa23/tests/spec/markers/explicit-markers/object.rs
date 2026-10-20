struct W(*const u8);unsafe impl Send for W{}fn f(x:&W){let _: &dyn Send=x;}
fn main(){}
