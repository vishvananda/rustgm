fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn sub(a:u8,b:&u8)->u8{<u8 as core::ops::Sub<&u8>>::sub(a,b)}fn main(){let _=sub(0,&1);}
