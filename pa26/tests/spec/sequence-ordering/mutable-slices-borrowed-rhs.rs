fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
use core::cmp::Ordering;fn rank(x:Option<Ordering>)->i8{match x{None=>2,Some(Ordering::Less)=>-1,Some(Ordering::Equal)=>0,Some(Ordering::Greater)=>1}}
fn main(){let mut a=[1u8,2];let mut b=[1u8,3];let x:&mut[u8]=&mut a;let y:&mut[u8]=&mut b;check(x<y);check(x.cmp(&y)==Ordering::Less);x[0]=4;check(x[0]==4);}