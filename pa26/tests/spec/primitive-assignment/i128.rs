fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn apply<T:core::ops::MulAssign>(a:&mut T,b:T){*a *= b;}fn main(){let mut a=-2i128;apply(&mut a,3i128);check(a==-6);}
