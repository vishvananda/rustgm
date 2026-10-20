trait A{}struct W<T>(T);impl<T>A for W<T>{}fn make<T>(x:T)->impl A{W(x)}fn need<T:Send>(_:T){}fn f<T:Send>(x:T){need(make(x));}
fn main(){}
