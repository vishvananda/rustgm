fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn apply<T:core::ops::BitAndAssign<R>,R>(a:&mut T,b:R){*a &= b;}fn main(){let mut a=10u64;apply(&mut a,6u64);check(a==2);}
