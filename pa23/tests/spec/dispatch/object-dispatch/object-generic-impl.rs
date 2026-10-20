fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{fn f(&self)->u8;}struct W<T>(T);impl<T:Copy>A for W<T>{fn f(&self)->u8{let _=self.0;7}}fn main(){let v=W(9u16);let p:&dyn A=&v;check(p.f()==7);}
