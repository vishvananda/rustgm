fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn apply<T:core::ops::AddAssign>(a:&mut T,b:T){*a+=b;}fn main(){let f:fn(&mut u32,u32)=apply::<u32>;let mut a=3;f(&mut a,4);check(a==7);}
