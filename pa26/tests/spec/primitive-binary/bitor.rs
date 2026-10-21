fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
use core::ops::BitOr;fn apply<T:core::ops::BitOr<R,Output=T>,R>(a:T,b:R)->T{a|b}fn main(){check(apply(6u8,3u8)==7);check(<u8 as BitOr>::bitor(6,3)==7);check(6u8.bitor(3u8)==7);let f:fn(u8,u8)->u8=<u8 as BitOr>::bitor;check(f(6,3)==7);}
