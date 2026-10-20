trait P{type A;}trait T:P{}fn check(_: &dyn T<A=u8>){}
fn main(){}
