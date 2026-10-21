fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn apply<T:core::ops::BitXorAssign>(a:&mut T,b:T){*a ^= b;}fn main(){let mut a=true;apply(&mut a,true);check(a==false);}
