trait A{type V;fn f(&self,x:Self::V)where Self::V:Copy{let a=x;let b=x;}}struct Q(u8);impl A for u8{type V=Q;}fn main(){let v=2u8;let p:&dyn A<V=Q>=&v;p.f(Q(1));}
