fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self+1}}fn main(){let p=A::f as fn(u8)->u8;let x=p(3u8);check(x==4);}