trait P{type A;}trait T:P<A=u8>{}fn f(_: &dyn T<A=u8>){}
fn main(){}
