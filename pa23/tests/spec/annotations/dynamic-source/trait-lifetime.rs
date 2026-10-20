trait T<'a>:'a{}fn check<'a>(_: *const dyn T<'a>){}
fn main(){}
