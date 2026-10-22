fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
use core::cmp::Ordering;fn rank(x:Option<Ordering>)->i8{match x{None=>2,Some(Ordering::Less)=>-1,Some(Ordering::Equal)=>0,Some(Ordering::Greater)=>1}}
fn main(){let a:[u8;0]=[];let b:[u8;0]=[];check(rank(a.partial_cmp(&b))==0);check(a.cmp(&b)==Ordering::Equal);check(a<=b);}