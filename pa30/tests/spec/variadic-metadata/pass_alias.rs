fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){check(unsafe{va_bridge::total(2,11i64,13i64)}==24);check(va_bridge::fixed(4)==5);}
