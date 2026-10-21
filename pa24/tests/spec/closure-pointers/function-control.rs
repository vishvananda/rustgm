fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn add(x:u8)->u8{x+1}const F:fn(u8)->u8=add;fn main(){check(F(7)==8);}