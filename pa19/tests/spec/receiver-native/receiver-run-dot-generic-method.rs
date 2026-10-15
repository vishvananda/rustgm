fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{fn f<T>(self,x:T)->T;}impl A for u8{fn f<T>(self,x:T)->T{x}}fn main(){let x=3u8.f::<u16>(7);check(x==7);}