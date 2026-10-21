fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn add<T:core::ops::Add<Output=T>>(a:T,b:T)->T{a+b}fn main(){let _=add(255u8,1u8);}
