fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn apply<T:core::ops::RemAssign>(a:&mut T,b:T){*a %= b;}fn main(){let mut a=8.0f64;apply(&mut a,3.0f64);check(a==2.0);}
