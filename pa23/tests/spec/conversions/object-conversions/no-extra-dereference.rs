trait T{}impl T for u8{}fn f<'a>(x:&'a &u8)->&'a dyn T{x}
fn main(){}
