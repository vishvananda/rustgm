fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
use core::marker::Copy as C;fn twice<T:C>(x:T)->(T,T){(x,x)}fn main(){check(twice(7u8).1==7);}
