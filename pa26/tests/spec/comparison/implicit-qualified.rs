fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){check(u8::eq(&3,&3));check(u8::ne(&3,&4));}
