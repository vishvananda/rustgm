fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A:Sized{fn f(self)->u8{9}}impl A for u8{fn f(self)->u8{self+1}}fn main(){let x=A::f(3u8);check(x==4);}