fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
use core::cmp::Ordering as O;
fn main(){let _:fn(&mut O,&O)->bool=O::eq;}