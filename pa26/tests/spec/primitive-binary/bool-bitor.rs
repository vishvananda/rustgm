fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){check(<bool as core::ops::BitOr>::bitor(false,true)==true);}
