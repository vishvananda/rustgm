trait P<'a>{type A;}trait T:for<'a>P<'a>{}impl<'a>P<'a> for u8{type A=&'a u8;}impl T for u8{}fn f(x:&u8)->&dyn T<A=&'static u8>{x}
fn main(){}
