trait P<'a>{type A;}trait T<'a>:P<'static>{}fn f(_: &dyn for<'a>T<'a,A=&'a u8>){}
fn main(){}
