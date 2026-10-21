fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){check(<i32 as core::ops::Shr<u128>>::shr(32,3)==4);}
