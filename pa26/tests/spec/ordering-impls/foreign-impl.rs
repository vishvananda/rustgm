fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
use core::cmp::Ordering as O;
impl Ord for O{fn cmp(&self,_:&Self)->O{O::Equal}}fn main(){}