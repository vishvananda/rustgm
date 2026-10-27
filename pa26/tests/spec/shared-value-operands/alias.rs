fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
use core::ops::{Add as Sum,Sub as Difference};type Byte=u8;fn main(){let b=5u8;check(<Byte as Sum<&Byte>>::add(2,&b)==7);check(<Byte as Difference<&Byte>>::sub(9,&b)==4);}
