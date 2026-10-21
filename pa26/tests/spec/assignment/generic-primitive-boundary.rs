fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn apply<T:core::ops::AddAssign<u32>>(x:&mut T){*x+=4u32;}fn main(){let mut x=3u32;apply(&mut x);check(x==7);}
