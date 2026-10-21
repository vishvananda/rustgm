fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
use core::cmp::Ordering as O;
fn main(){let a:for<'a,'b> fn(&'a O,&'b O)->bool=O::eq;let b:for<'a,'b> fn(&'a O,&'b O)->O=O::cmp;let c:for<'a,'b> fn(&'a O,&'b O)->Option<O>=O::partial_cmp;check(a(&O::Equal,&O::Equal));check(b(&O::Greater,&O::Less) as i8==1);check(match c(&O::Equal,&O::Less){Some(O::Greater)=>true,_=>false});}