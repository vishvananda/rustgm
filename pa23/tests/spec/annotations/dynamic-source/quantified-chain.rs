trait P<'a,'b>{type A;}trait Q<'a>:for<'b>P<'a,'b>{}trait T:for<'a>Q<'a>{}fn f(_: &dyn T<A=u8>){}
fn main(){}
