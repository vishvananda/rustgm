fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
use core::cmp::Ordering as O;
fn f<T:Ord>(a:T,b:T)->O{a.cmp(&b)}fn main(){check(f(O::Greater,O::Equal) as i8==1);}