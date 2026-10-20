trait P{type A;}trait T:P<A=u16>{}impl P for u8{type A=u16;}impl T for u8{}fn f(x:&u8)->&dyn T{x}
fn main(){}
