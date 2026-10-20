trait T<'a>:'a{}fn f<'a>(_: *const dyn T<'a>){}
fn main(){}
