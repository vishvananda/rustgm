trait P{type A;fn get(&self)->Self::A;}trait L:P<A=u8>{}trait R:P<A=u8>{}trait T:L+R{}fn f(x:&dyn T)->u8{x.get()}
fn main(){}
