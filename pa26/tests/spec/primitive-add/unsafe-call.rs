fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let f:unsafe fn(u8,u8)->u8=<u8 as core::ops::Add>::add;let _=f(2,5);}
