trait T<'a>:'a{}type A=dyn for<'a> T<'a>;
fn main(){}
