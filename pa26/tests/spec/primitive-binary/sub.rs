fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
use core::ops::Sub;fn apply<T:core::ops::Sub<R,Output=T>,R>(a:T,b:R)->T{a-b}fn main(){check(apply(9u8,2u8)==7);check(<u8 as Sub>::sub(9,2)==7);check(9u8.sub(2u8)==7);let f:fn(u8,u8)->u8=<u8 as Sub>::sub;check(f(9,2)==7);}
