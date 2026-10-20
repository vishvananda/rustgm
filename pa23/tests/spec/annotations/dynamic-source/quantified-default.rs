trait T<'a>:'a{}fn f(_: *const dyn for<'a> T<'a>){}
fn main(){}
