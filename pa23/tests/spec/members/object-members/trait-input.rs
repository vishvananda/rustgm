trait T<A>{}fn need<X:T<u8>+?Sized>(_:&X){}fn f(x:&dyn T<u8>){need(x)}
fn main(){}
