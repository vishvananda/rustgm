trait P<'a>{type A;}trait T:for<'a>P<'a,A=&'a u8>{}fn f(_: &dyn T){}
fn main(){}
