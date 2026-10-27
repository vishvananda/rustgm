fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let a=2u8;let b=5u8;check(<&u8 as core::ops::Add<&u8>>::add(&a,&b)==7);}
