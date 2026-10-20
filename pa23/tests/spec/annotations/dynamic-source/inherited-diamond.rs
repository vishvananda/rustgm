trait P{type A;}trait L:P<A=u8>{}trait R:P<A=u8>{}trait T:L+R{}fn f(_: &dyn T){}
fn main(){}
