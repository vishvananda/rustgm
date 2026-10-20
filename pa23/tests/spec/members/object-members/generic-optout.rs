trait T{fn get<A>(&self,x:A) where Self:Sized;}fn f(x:&dyn T){x.get(7u8)}
fn main(){}
