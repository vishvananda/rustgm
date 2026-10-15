fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A:Sized{fn f(self)->u8{9}}impl A for u8{}fn main(){let x=<u8 as A>::f(3);check(x==9);}