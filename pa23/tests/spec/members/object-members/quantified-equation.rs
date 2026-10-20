trait T<'a>{type A;fn get(&self,x:&'a u8)->Self::A;}fn f<'a>(x:&dyn for<'b>T<'b,A=&'b u8>,y:&'a u8)->&'a u8{x.get(y)}
fn main(){}
