trait T<'a>{type A;fn get(&self,x:&'a u8)->Self::A;}fn f(x:&dyn for<'b>T<'b,A=&'b u8>,y:&u8)->&'static u8{x.get(y)}
fn main(){}
