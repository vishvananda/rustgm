trait A{}struct W<T>(T);impl<T>A for W<T>{}fn make<T>(x:T)->impl A{W(x)}fn need<T:Send>(_:T){}fn f(){need(make(7u8));}
fn main(){}
