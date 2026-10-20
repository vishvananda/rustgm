trait T<'a>{type A;}impl<'a>T<'a> for u8{type A=&'a u8;}fn f(x:&u8)->&dyn for<'a>T<'a,A=&'a u8>{x}
fn main(){}
