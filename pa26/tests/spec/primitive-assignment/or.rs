fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn apply<T:core::ops::BitOrAssign<R>,R>(a:&mut T,b:R){*a |= b;}fn main(){let mut a=10u32;apply(&mut a,6u32);check(a==14);}
