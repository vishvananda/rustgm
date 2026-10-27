fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let a:fn(u8,&'static u8)->u8=<u8 as core::ops::Add<&'static u8>>::add;let s:fn(u8,&'static u8)->u8=<u8 as core::ops::Sub<&'static u8>>::sub;check(a(2,&5)==7);check(s(9,&5)==4);}
