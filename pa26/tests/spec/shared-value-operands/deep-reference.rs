fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let b=5u8;let r=&b;let _=<u8 as core::ops::Sub<&&u8>>::sub(9,&r);}
