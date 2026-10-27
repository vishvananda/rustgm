fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let mut b=5u8;check(<u8 as core::ops::Add<&u8>>::add(2,&mut b)==7);b=3;check(<u8 as core::ops::Sub<&u8>>::sub(9,&mut b)==6);b=7;check(b==7);}
