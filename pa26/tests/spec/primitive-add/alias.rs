fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
use core::ops::Add as Sum;fn main(){check(<u8 as Sum>::add(2,5)==7);}
