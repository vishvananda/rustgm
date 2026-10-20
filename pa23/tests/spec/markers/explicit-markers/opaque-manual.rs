trait A{}struct W(*const u8);impl A for W{}unsafe impl Send for W{}fn make(x:*const u8)->impl A{W(x)}fn need<T:Send>(_:T){}fn f(x:*const u8){need(make(x));}
fn main(){}
