fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
use core::ops::BitXor;fn apply<T:core::ops::BitXor<R,Output=T>,R>(a:T,b:R)->T{a^b}fn main(){check(apply(6u8,3u8)==5);check(<u8 as BitXor>::bitxor(6,3)==5);check(6u8.bitxor(3u8)==5);let f:fn(u8,u8)->u8=<u8 as BitXor>::bitxor;check(f(6,3)==5);}
