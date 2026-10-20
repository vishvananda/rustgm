trait T<'a>:'a{}fn f<'a:'a>(_: &dyn T<'a>){}
fn main(){}
