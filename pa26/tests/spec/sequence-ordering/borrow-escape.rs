fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
use core::cmp::Ordering;fn rank(x:Option<Ordering>)->i8{match x{None=>2,Some(Ordering::Less)=>-1,Some(Ordering::Equal)=>0,Some(Ordering::Greater)=>1}}
fn f<'a>(a:&'a[u8],b:&'a[u8])->&'static[u8]{let _=a<b;a}fn main(){}