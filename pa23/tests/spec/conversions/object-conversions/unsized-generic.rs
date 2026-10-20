trait T{}fn f<X:T+?Sized>(x:&X)->&dyn T{x}
fn main(){}
