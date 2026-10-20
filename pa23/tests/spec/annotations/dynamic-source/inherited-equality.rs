trait P{type A;}trait T:P<A=u8>{}fn f(_: &dyn T){}
fn main(){}
