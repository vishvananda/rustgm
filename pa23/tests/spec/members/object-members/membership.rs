trait T{}fn need<X:T+?Sized>(_:&X){}fn f(x:&dyn T){need(x)}
fn main(){}
