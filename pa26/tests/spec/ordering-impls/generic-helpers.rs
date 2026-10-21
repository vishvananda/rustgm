fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
use core::cmp::Ordering as O;
fn f<T:Ord>(a:T,b:T,c:T)->T{a.min(b).max(c)}fn g<T:Ord>(a:T,b:T,c:T)->T{a.clamp(b,c)}fn main(){check(f(O::Equal,O::Less,O::Greater) as i8==1);check(g(O::Less,O::Equal,O::Greater) as i8==0);check(g(O::Equal,O::Equal,O::Equal) as i8==0);}