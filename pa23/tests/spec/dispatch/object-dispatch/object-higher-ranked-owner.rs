fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A<'a>{fn f(&self,x:&'a u8)->&'a u8;}impl<'a>A<'a>for u8{fn f(&self,x:&'a u8)->&'a u8{x}}fn main(){let v=7u8;let x=9u8;let p:&dyn for<'a>A<'a>=&v;check(*p.f(&x)==9);}
