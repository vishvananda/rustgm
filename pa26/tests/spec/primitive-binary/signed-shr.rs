fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){check(<i16 as core::ops::Shr<i16>>::shr(-16,2)==-4);}
