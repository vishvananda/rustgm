trait T{type A;fn f(&self)->Self::A;}fn check(_: &dyn T<A=u8>){}
fn main(){}
