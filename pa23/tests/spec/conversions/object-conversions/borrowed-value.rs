trait T{}impl<'a>T for &'a u8{}fn f<'a,'b>(x:&'a &'b u8)->&'a dyn T{x}
fn main(){}
