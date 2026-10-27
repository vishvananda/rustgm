fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let p=&char::MAX;check(*p=='\u{10ffff}');{let p=&mut char::MIN;*p='q';check(*p=='q');}check(char::MIN=='\0');}
