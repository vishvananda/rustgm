fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn apply<T:core::ops::AddAssign>(a:&mut T,b:T){*a += b;}fn main(){let mut a=2.0f32;apply(&mut a,3.0f32);check(a==5.0);}
