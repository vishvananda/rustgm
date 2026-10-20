trait T<'a>{}fn need<X:for<'a>T<'a>+?Sized>(_:&X){}fn f(x:&dyn for<'a>T<'a>){need(x)}
fn main(){}
