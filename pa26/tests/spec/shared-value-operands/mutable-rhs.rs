fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let mut b=5u8;let _=<u8 as core::ops::Add<&mut u8>>::add(2,&mut b);}
