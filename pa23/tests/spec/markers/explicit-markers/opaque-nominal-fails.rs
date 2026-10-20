fn f(x:*const u8){let _=W(make(x));}struct W<T:Send>(T);trait A{}struct P(*const u8);impl A for P{}fn make(x:*const u8)->impl A{P(x)}
fn main(){}
