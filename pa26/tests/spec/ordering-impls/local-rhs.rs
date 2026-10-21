fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
use core::cmp::Ordering as O;
struct Local(i8);impl PartialEq<Local> for O{fn eq(&self,o:&Local)->bool{*self as i8==o.0}}fn main(){check(O::Less==Local(-1));check(O::Equal.eq(&Local(0)));check(O::Greater!=Local(0));}