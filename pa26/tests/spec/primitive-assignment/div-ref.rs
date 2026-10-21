fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn apply<T:core::ops::DivAssign<R>,R>(a:&mut T,b:R){*a /= b;}fn main(){let mut a=8i32;let b=2i32;apply(&mut a,&b);check(a==4);}
