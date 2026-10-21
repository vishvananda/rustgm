fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
use core::cmp::Ordering as O;
const L:O=O::Less;fn f(a:O)->bool{match a{L=>true,_=>false}}fn main(){check(f(O::Less));check(!f(O::Equal));}