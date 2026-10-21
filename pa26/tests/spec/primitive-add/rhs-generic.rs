fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn add<T:core::ops::Add<R,Output=T>,R>(a:T,b:R)->T{a+b}fn main(){check(add(2u32,5u32)==7);}
