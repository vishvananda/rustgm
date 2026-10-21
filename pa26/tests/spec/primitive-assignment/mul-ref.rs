fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn apply<T:core::ops::MulAssign<R>,R>(a:&mut T,b:R){*a *= b;}fn main(){let mut a=8u8;let b=2u8;apply(&mut a,&b);check(a==16);}
