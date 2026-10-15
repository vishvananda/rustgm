fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A:Sized{fn f(self)->u8{A::g(self)+1}fn g(self)->u8;}impl A for u8{fn g(self)->u8{self+2}}fn main(){let x=A::f(3u8);check(x==6);}