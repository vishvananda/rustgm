trait T<'a,'b>:'a+'b{}fn f<'a:'a,'b:'b>(_: *const dyn T<'a,'b>){}
fn main(){}
