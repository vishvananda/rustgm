fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let mut n=2u8;let _=<u8 as core::ops::Sub<&mut u8>>::sub(7,&mut n);}
