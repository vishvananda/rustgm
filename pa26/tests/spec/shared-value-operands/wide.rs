fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let b=18446744073709551616u128;check(<u128 as core::ops::Add<&u128>>::add(b,&b)==36893488147419103232);check(<u128 as core::ops::Sub<&u128>>::sub(b+1,&b)==1);let n=2i128;check(<i128 as core::ops::Sub<&i128>>::sub(-7,&n)==-9);}
