trait P<'a>{type A;}trait T:for<'a>P<'a>{}fn f(_: &dyn T){}
fn main(){}
