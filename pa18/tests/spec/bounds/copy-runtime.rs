fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn twice<T:Copy>(x:T)->(T,T){(x,x)}fn main(){let (a,b)=twice(300u16);check(a+b==600);}
