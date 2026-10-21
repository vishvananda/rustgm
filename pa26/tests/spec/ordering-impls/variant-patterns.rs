fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
use core::cmp::Ordering as O;
fn rank(x:O)->i8{match x{O::Less=>-1,O::Equal=>0,O::Greater=>1}}fn main(){check(rank(O::Less)==-1);check(rank(O::Equal)==0);check(rank(O::Greater)==1);}