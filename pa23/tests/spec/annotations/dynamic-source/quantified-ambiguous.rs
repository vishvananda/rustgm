trait P<'a>{type A;}trait Q<'a>{type A;}trait T:for<'a>P<'a>+for<'a>Q<'a>{}fn f(_: &dyn T<A=u8>){}
fn main(){}
