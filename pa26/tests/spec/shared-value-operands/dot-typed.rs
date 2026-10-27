fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
use core::ops::{Add,Sub};fn main(){let b=5u8;check(2u8.add(&b)==7);check(9u8.sub(&b)==4);}
