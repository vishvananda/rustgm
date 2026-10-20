fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A<T>{fn f(&self,x:T)->T;}impl A<u16>for u8{fn f(&self,x:u16)->u16{x+2}}fn main(){let v=7u8;let p:&dyn A<u16>=&v;check(p.f(1000)==1002);}
