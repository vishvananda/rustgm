trait T<'a>:'a{}fn f<'a:'a>(_: *const dyn T<'a>){}
fn main(){}
