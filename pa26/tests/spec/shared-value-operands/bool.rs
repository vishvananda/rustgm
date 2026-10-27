fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let _=<bool as core::ops::Add<&bool>>::add(true,&false);}
