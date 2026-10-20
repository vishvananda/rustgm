trait P{type A;}trait T:P<A=u8>{}fn need<X:P<A=u8>+?Sized>(_:&X){}fn f(x:&dyn T){need(x)}
fn main(){}
