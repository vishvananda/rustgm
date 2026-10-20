trait P<'a,'b>{type A;}trait T<'a>:for<'b>P<'a,'b>{}impl<'a,'b>P<'a,'b> for u8{type A=&'b u8;}impl<'a>T<'a> for u8{}fn f(x:&u8)->&dyn for<'a>T<'a,A=&'a u8>{x}
fn main(){}
