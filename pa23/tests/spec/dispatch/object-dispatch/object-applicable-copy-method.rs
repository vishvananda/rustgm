fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{type V;fn f(&self,x:Self::V)->u8 where Self::V:Copy{let a=x;let b=x;9}}impl A for u8{type V=u16;}fn main(){let v=2u8;let p:&dyn A<V=u16>=&v;check(p.f(3)==9);}
