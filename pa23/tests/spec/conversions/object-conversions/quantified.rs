trait T<'a>{}impl<'a>T<'a> for u8{}fn f(x:&u8)->&dyn for<'a>T<'a>{x}
fn main(){}
