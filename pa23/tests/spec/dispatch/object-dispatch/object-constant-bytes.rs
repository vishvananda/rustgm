fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{fn f(&self)->u8;}impl A for [u8;2]{fn f(&self)->u8{self[1]}}const P:&dyn A=b"ab";fn main(){check(P.f()==98);}
