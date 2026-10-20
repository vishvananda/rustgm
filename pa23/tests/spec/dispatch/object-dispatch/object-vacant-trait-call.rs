trait E{}trait A{type V;fn f(&self)where Self::V:E{}}impl A for u8{type V=u16;}fn main(){let v=2u8;let p:&dyn A<V=u16>=&v;p.f();}
