trait T{type A;fn set(&self,x:Self::A);}fn f(x:&dyn T<A=u8>){x.set(7u8)}
fn main(){}
