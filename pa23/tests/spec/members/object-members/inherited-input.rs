trait P<A>{}trait T:P<u8>{}fn need<X:P<u8>+?Sized>(_:&X){}fn f(x:&dyn T){need(x)}
fn main(){}
