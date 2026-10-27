fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const LOW:char=char::MIN;const HIGH:char=char::MAX;const fn high()->char{HIGH}fn main(){check(LOW=='\0');check(high()=='\u{10ffff}');check(const{char::MAX as u32}==1114111);}
