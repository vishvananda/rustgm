trait P<'a>{type A;}trait T:for<'a>P<'a>{}fn f<'b>(_: &dyn T<A=&'b u8>){}
fn main(){}
