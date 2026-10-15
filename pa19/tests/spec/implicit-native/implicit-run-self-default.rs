fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A:Sized{fn f(self)->u8;fn g(self)->u8{Self::f(self)+1}}impl A for u8{fn f(self)->u8{self}}fn main(){let x=u8::g(3);check(x==4);}