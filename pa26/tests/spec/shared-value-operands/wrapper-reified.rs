fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn add(a:u8,b:&u8)->u8{<u8 as core::ops::Add<&u8>>::add(a,b)}fn sub(a:u8,b:&u8)->u8{<u8 as core::ops::Sub<&u8>>::sub(a,b)}fn main(){let a:fn(u8,&u8)->u8=add;let s:fn(u8,&u8)->u8=sub;let b=5u8;check(a(2,&b)==7);check(s(9,&b)==4);}
