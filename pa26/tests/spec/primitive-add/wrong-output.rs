fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn add<T:core::ops::Add<Output=u16>>(a:T,b:T)->u16{a+b}fn main(){let _=add(2u8,5u8);}
