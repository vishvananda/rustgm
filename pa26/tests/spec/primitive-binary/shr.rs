fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
use core::ops::Shr;fn apply<T:core::ops::Shr<R,Output=T>,R>(a:T,b:R)->T{a>>b}fn main(){check(apply(12u8,2u8)==3);check(<u8 as Shr>::shr(12,2)==3);check(12u8.shr(2u8)==3);let f:fn(u8,u8)->u8=<u8 as Shr>::shr;check(f(12,2)==3);}
