fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn apply<T:core::ops::AddAssign<R>,R>(a:&mut T,b:R){*a += b;}fn main(){let mut a=8u32;apply(&mut a,2u32);check(a==10);}
