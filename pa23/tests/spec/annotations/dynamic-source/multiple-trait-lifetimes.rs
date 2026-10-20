trait T<'a,'b>:'a+'b{}fn check<'a,'b>(_: *const dyn T<'a,'b>){}
fn main(){}
