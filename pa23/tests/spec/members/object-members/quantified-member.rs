trait T<'a>{fn get(&self,x:&'a u8)->u8;}fn f(x:&dyn for<'a>T<'a>, y:&u8)->u8{x.get(y)}
fn main(){}
