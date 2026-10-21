fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
use core::cmp::Ordering as O;
struct Local(i8);impl PartialEq<Local> for O{fn eq(&self,o:&Local)->bool{*self as i8==o.0}}impl PartialOrd<Local> for O{fn partial_cmp(&self,o:&Local)->Option<O>{(*self as i8).partial_cmp(&o.0)}}fn main(){check(O::Less < Local(0));check(O::Greater > Local(-1));}