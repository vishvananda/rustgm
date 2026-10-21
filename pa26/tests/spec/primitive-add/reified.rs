fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let f:fn(u8,u8)->u8=<u8 as core::ops::Add<u8>>::add;check(f(2,5)==7);}
