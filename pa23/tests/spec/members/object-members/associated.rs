trait T{type A;}fn need<X:T<A=u8>+?Sized>(_:&X){}fn f(x:&dyn T<A=u8>){need(x)}
fn main(){}
