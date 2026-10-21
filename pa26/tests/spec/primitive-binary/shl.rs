fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
use core::ops::Shl;fn apply<T:core::ops::Shl<R,Output=T>,R>(a:T,b:R)->T{a<<b}fn main(){check(apply(3u8,2u8)==12);check(<u8 as Shl>::shl(3,2)==12);check(3u8.shl(2u8)==12);let f:fn(u8,u8)->u8=<u8 as Shl>::shl;check(f(3,2)==12);}
