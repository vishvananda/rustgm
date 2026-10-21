fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
use core::ops::Mul;fn apply<T:core::ops::Mul<R,Output=T>,R>(a:T,b:R)->T{a*b}fn main(){check(apply(3u8,4u8)==12);check(<u8 as Mul>::mul(3,4)==12);check(3u8.mul(4u8)==12);let f:fn(u8,u8)->u8=<u8 as Mul>::mul;check(f(3,4)==12);}
