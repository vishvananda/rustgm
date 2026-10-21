fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
use core::cmp::Ordering as O;
fn main(){let a=O::Less;let b=O::Greater;check(a.eq(&a));check(a.ne(&b));check(a.lt(&b));check(a.le(&a));check(b.gt(&a));check(b.ge(&b));check(a.cmp(&b) as i8 == -1);check(match a.partial_cmp(&b){Some(O::Less)=>true,_=>false});}