fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
use core::ops::BitAnd;fn apply<T:core::ops::BitAnd<R,Output=T>,R>(a:T,b:R)->T{a&b}fn main(){check(apply(6u8,3u8)==2);check(<u8 as BitAnd>::bitand(6,3)==2);check(6u8.bitand(3u8)==2);let f:fn(u8,u8)->u8=<u8 as BitAnd>::bitand;check(f(6,3)==2);}
