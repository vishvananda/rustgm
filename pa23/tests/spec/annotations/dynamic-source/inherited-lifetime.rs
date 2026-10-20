trait P<'a>{type A;}trait T<'a>:P<'a,A=&'a u8>{}fn f<'a>(_: &dyn T<'a>){}
fn main(){}
