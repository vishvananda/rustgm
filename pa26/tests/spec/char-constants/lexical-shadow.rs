fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
#[allow(non_camel_case_types)]struct char;impl char{const MIN:u8=7;const MAX:u8=9;}fn main(){check(char::MIN==7);check(char::MAX==9);check(core::primitive::char::MIN=='\0');check(core::primitive::char::MAX=='\u{10ffff}');}
