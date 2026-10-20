trait T<'a>{}fn need<X:for<'a>T<'a>+?Sized>(_:&X){}fn f(x:&dyn T<'static>){need(x)}
fn main(){}
