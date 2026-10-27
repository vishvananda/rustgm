fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn add(a:u8,b:&u8)->u8{<u8 as core::ops::Add<&u8>>::add(a,b)}fn main(){let _=add(255,&1);}
