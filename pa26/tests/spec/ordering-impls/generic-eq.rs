fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
use core::cmp::Ordering as O;
fn f<T:Eq>(a:T,b:T)->bool{a==b}fn main(){check(f(O::Less,O::Less));check(!f(O::Equal,O::Greater));}