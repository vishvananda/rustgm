trait T{type A;fn get(&self)->Self::A;}fn f(x:&dyn T<A=u8>)->u8{x.get()}
fn main(){}
