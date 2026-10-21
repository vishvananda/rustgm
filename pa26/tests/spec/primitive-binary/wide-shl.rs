fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){check(<i128 as core::ops::Shl<u8>>::shl(1,65)==36893488147419103232);}
