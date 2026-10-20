fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{fn f(&mut self,n:u8)->u8;}impl A for u8{fn f(&mut self,n:u8)->u8{*self+=n;*self}}fn main(){let mut v=7u8;let p:&mut dyn A=&mut v;check(p.f(2)==9);check(v==9);}
