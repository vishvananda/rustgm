fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
use core::ops::Rem;fn apply<T:core::ops::Rem<R,Output=T>,R>(a:T,b:R)->T{a%b}fn main(){check(apply(9u8,2u8)==1);check(<u8 as Rem>::rem(9,2)==1);check(9u8.rem(2u8)==1);let f:fn(u8,u8)->u8=<u8 as Rem>::rem;check(f(9,2)==1);}
