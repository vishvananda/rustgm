trait P{type A;fn get(&self)->Self::A;}trait T:P<A=u8>{}fn f(x:&dyn T)->u8{x.get()}
fn main(){}
