fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){check(char::MAX=='\u{10ffff}');check(char::MAX as u32==1114111);}
