trait P{}trait T:P{}fn need<X:P+?Sized>(_:&X){}fn f(x:&dyn T){need(x)}
fn main(){}
