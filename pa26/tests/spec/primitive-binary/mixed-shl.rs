fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){check(<u16 as core::ops::Shl<i8>>::shl(3,2)==12);}
