fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
use core::cmp::PartialEq as E;fn same<A:?Sized+E<B>,B:?Sized>(a:&A,b:&B)->bool{a==b}fn main(){check(same(&3u8,&3u8));}
