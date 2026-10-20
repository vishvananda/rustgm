trait A{}struct W(*const u8);impl A for W{}fn f(x:&W){let _: &(dyn A+Send)=x;}
fn main(){}
