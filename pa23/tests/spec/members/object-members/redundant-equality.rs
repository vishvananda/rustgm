trait P{type A;fn get(&self)->Self::A;}trait T:P<A=u8>{}fn f(x:&dyn T<A=u8>)->u8{x.get()}
fn main(){}
