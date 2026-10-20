trait P<'a>{fn get(&self,x:&'a u8)->u8;}trait T:for<'a>P<'a>{}fn f(x:&dyn T,y:&u8)->u8{x.get(y)}
fn main(){}
