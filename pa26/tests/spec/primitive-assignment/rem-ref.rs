fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn apply<T:core::ops::RemAssign<R>,R>(a:&mut T,b:R){*a %= b;}fn main(){let mut a=8u16;let b=3u16;apply(&mut a,&b);check(a==2);}
