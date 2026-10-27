fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){check(<char>::MAX=='\u{10ffff}');check(core::primitive::char::MIN=='\0');check(std::primitive::char::MAX=='\u{10ffff}');}
