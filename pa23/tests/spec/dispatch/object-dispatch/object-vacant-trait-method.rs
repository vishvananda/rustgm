fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait E{}trait A{type V;fn g(&self)->u8{7}fn f(&self)->u8 where Self::V:E{9}}impl A for u8{type V=u16;}fn main(){let v=2u8;let p:&dyn A<V=u16>=&v;check(p.g()==7);}
