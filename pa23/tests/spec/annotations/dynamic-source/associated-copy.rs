struct N(u8);trait T{type A:Copy;}fn f(_: &dyn T<A=N>){}
fn main(){}
