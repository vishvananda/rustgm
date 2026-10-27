fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let b=5u16;let _=<u8 as core::ops::Add<&u16>>::add(2,&b);}
