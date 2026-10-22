fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
use core::cmp::Ordering;fn rank(x:Option<Ordering>)->i8{match x{None=>2,Some(Ordering::Less)=>-1,Some(Ordering::Equal)=>0,Some(Ordering::Greater)=>1}}
fn main(){let f:for<'a,'b>fn(&'a[u8;2],&'b[u8;2])->Option<Ordering>=<[u8;2]as PartialOrd>::partial_cmp;check(rank(f(&[1,2],&[1,3]))==-1);}