fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
use core::ops::Add;fn main(){check(2u8.add(5u8)==7);}
