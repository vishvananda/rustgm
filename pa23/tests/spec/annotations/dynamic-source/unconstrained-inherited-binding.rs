trait P<'a>{type A;}trait T:for<'a>P<'a>{}fn f(_: &dyn for<'x>T<A=&'x u8>){}
fn main(){}
