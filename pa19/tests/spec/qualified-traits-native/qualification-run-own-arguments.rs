fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{fn f<T>(self,x:T)->T;}impl A for u8{fn f<U>(self,x:U)->U{x}}fn main(){let x=<u8 as A>::f::<u16>(3,7);check(x==7);}