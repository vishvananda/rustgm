fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let mut b=5u8;let r=&b;b=7;check(<u8 as core::ops::Add<&u8>>::add(2,r)==7);}
