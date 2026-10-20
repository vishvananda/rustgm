trait A{}struct W<T>(T);impl<T>A for W<T>{}fn make<T>(x:T)->impl A{W(x)}fn need<T:Send>(_:T){}fn f(x:*const u8){need(make(x));}
fn main(){}
