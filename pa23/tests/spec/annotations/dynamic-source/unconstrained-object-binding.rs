trait T{type A;}fn f(_: &dyn for<'a>T<A=&'a u8>){}
fn main(){}
