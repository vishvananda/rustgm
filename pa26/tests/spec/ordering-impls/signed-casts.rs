fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
use core::cmp::Ordering as O;
fn main(){check(O::Less as i8 == -1);check(O::Equal as i8 == 0);check(O::Greater as i8 == 1);check(O::Less < O::Equal);}