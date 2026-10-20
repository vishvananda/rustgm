fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{fn f(&self)->u8;}trait B{fn g(&self)->u8;}impl A for u8{fn f(&self)->u8{*self}}impl B for dyn A+'_ {fn g(&self)->u8{self.f()+2}}fn main(){let v=7u8;let p:&dyn A=&v;check(p.g()==9);}
