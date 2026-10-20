trait T{}trait U{}fn need<X:U+?Sized>(_:&X){}fn f(x:&dyn T){need(x)}
fn main(){}
