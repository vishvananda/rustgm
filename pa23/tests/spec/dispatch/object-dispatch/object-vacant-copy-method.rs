fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{type V;fn g(&self)->u8{7}fn f(&self,x:Self::V)->u8 where Self::V:Copy{let a=x;let b=x;9}}struct Q(u8);impl A for u8{type V=Q;}fn main(){let v=2u8;let p:&dyn A<V=Q>=&v;check(p.g()==7);}
