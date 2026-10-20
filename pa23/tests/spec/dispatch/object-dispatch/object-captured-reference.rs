fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{fn f(&self)->&u8;}struct W<'a>(&'a u8);impl<'a>A for W<'a>{fn f(&self)->&u8{self.0}}fn main(){let v=7u8;let w=W(&v);let p:&dyn A=&w;check(*p.f()==7);}
