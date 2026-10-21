fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f<T:core::ops::Shl<i8,Output=u16>>(x:T)->u16{x<<1i8}fn main(){let _=f(2u8);}
