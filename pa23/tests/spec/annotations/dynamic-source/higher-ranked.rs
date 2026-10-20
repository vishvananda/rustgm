trait T<'a>{fn f(&self,x:&'a u8);}fn check(_: &dyn for<'a> T<'a>){}
fn main(){}
