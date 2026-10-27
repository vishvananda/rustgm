fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){check(u8::MAX==255);check(i8::MAX==127);check(u128::MAX==340282366920938463463374607431768211455);check(char::MAX as u8==255);}
