fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){check(<u8 as core::ops::Add<u8>>::add(2,5)==7);}
