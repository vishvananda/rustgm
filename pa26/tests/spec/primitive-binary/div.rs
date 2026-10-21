fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
use core::ops::Div;fn apply<T:core::ops::Div<R,Output=T>,R>(a:T,b:R)->T{a/b}fn main(){check(apply(9u8,2u8)==4);check(<u8 as Div>::div(9,2)==4);check(9u8.div(2u8)==4);let f:fn(u8,u8)->u8=<u8 as Div>::div;check(f(9,2)==4);}
