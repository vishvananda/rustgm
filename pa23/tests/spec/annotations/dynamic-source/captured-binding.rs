trait T{type A;}fn check<'a>(_: &'a dyn T<A=&'a u8>){}
fn main(){}
