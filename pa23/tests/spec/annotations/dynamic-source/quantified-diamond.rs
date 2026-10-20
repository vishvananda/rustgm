trait P<'a>{type A;}trait L:for<'a>P<'a>{}trait R:for<'a>P<'a>{}trait T:L+R{}fn f(_: &dyn T<A=u8>){}
fn main(){}
