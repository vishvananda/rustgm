trait P<'a,'b>{type A;}trait T<'a>:for<'b>P<'a,'b>{}fn f(_: &dyn for<'a>T<'a,A=&'a u8>){}
fn main(){}
