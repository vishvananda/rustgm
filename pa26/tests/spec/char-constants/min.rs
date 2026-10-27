fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){check(char::MIN=='\0');check(char::MIN as u32==0);}
