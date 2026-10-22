fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
use core::cmp::Ordering;fn rank(x:Option<Ordering>)->i8{match x{None=>2,Some(Ordering::Less)=>-1,Some(Ordering::Equal)=>0,Some(Ordering::Greater)=>1}}
fn main(){check(rank(<[u8;2] as PartialOrd>::partial_cmp(&[1,2],&[1,3]))==-1);check(<[u8;2] as Ord>::cmp(&[1,2],&[1,3])==Ordering::Less);}