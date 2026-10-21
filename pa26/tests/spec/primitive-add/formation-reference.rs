fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn add<'a,T:core::ops::Add<Output=T>>(a:T,b:T,r:&'a u8)->(T,&'a u8){(a+b,r)}fn main(){let n=9;let p=add(2u8,5u8,&n);check(p.0==7&&*p.1==9);}
