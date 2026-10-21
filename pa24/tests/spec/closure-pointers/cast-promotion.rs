fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let f=&((|x:u8|x+1) as fn(u8)->u8);check(f(7)==8);}