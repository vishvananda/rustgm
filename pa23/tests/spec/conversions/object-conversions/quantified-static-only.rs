trait T<'a>{}impl T<'static> for u8{}fn f(x:&u8)->&dyn for<'a>T<'a>{x}
fn main(){}
