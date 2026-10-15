fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A<T=Self>{fn f(self,x:T)->T;}impl A for u8{fn f(self,x:u8)->u8{x}}fn main(){let x=<u8 as A>::f(3,7);check(x==7);}