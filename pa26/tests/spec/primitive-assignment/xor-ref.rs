fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn apply<T:core::ops::BitXorAssign<R>,R>(a:&mut T,b:R){*a ^= b;}fn main(){let mut a=10i32;let b=6i32;apply(&mut a,&b);check(a==12);}
