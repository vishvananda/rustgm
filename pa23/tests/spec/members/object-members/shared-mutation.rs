trait T{fn set(&mut self,x:u8);}fn f(x:&dyn T){x.set(4)}
fn main(){}
