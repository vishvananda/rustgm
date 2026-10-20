struct W<'a>(&'a u8);unsafe impl<'a:'static> Send for W<'a>{}fn need<T:Send>(_:T){}fn f<'a>(x:W<'a>){need(x);}
fn main(){}
