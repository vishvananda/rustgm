trait P<'a>{type A;}trait T:for<'a>P<'a>{}impl<'a>P<'a> for u8{type A=u16;}impl T for u8{}fn f(x:&u8)->&dyn T<A=u16>{x}
fn main(){}
