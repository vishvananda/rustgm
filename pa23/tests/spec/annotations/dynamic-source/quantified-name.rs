trait P<'a>{type A;}trait T:for<'a>P<'a>{}fn f(_: &dyn T<B=u8>){}
fn main(){}
