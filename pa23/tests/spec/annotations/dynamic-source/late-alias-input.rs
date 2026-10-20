trait T<'a>:'a{}type A<'a>=&'a u8;fn f<'a>(_:A<'a>)->*const dyn T<'a>{loop{}}
fn main(){}
