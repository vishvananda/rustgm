fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
type C=char;type D=C;fn main(){check(C::MIN=='\0');check(D::MAX=='\u{10ffff}');}
