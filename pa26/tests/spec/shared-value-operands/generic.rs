fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn add<'a,T:core::ops::Add<&'a u8,Output=u8>>(a:T,b:&'a u8)->u8{a+b}fn sub<'a,T:core::ops::Sub<&'a u8,Output=u8>>(a:T,b:&'a u8)->u8{a-b}fn main(){let b=5u8;check(add(2u8,&b)==7);check(sub(9u8,&b)==4);}
