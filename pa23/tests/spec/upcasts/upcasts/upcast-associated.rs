fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{type V;fn a(&self)->Self::V;}trait B:A<V=u16>{}impl A for u8{type V=u16;fn a(&self)->u16{1000}}impl B for u8{}fn main(){let v=7u8;let p:&dyn B=&v;let q:&dyn A<V=u16>=p;check(q.a()==1000);}
