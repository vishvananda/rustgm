pub fn f<'a>(x:&'a u8)->&'a u8{let f=|s:&u8|s;f(x)}
fn main(){}
