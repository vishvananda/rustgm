fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn left(t:&mut u32)->u8{*t=*t*10+1;7}fn right<'a>(t:&mut u32,n:&'a u8)->&'a u8{*t=*t*10+2;n}fn main(){let n=2;let mut order=0;check(<u8 as core::ops::Add<&u8>>::add(left(&mut order),right(&mut order,&n))==9);check(order==12);check(<u8 as core::ops::Sub<&u8>>::sub(left(&mut order),right(&mut order,&n))==5);check(order==1212);}
