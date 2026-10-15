fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A:Sized{fn f(self)->u8{7}}impl A for u8{}const P:fn(u8)->u8=A::f;fn main(){let x=P(3u8);check(x==7);}