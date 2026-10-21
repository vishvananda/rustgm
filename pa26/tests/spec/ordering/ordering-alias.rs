fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
use core::cmp as c;use core::sync::atomic::Ordering as A;fn main(){let o:c::Ordering=c::Ordering::Less;check(o as i8 == -1);check(match A::Acquire{A::Acquire=>true,_=>false});}
