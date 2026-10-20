trait P<'a>{type A;fn get(&self,x:&'a u8)->Self::A;}trait T:for<'a>P<'a,A=&'a u8>{}fn f<'a>(x:&dyn T,y:&'a u8)->&'a u8{x.get(y)}
fn main(){}
