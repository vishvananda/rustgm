trait T{type A;}impl T for u8{type A=u16;}fn f(x:&u8)->&dyn T<A=u16>{x}
fn main(){}
