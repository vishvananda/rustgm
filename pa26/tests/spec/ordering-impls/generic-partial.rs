fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
use core::cmp::Ordering as O;
fn f<T:PartialOrd>(a:T,b:T)->Option<O>{a.partial_cmp(&b)}fn main(){check(match f(O::Less,O::Greater){Some(O::Less)=>true,_=>false});}