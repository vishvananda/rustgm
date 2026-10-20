fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A<'a>{fn a(&self,x:&'a u8)->&'a u8;}impl<'a>A<'a>for u8{fn a(&self,x:&'a u8)->&'a u8{x}}fn main(){let v=7u8;let p:&dyn for<'a>A<'a>=&v;let q:&dyn A<'static>=p;check(*q.a(&9u8)==9);}
