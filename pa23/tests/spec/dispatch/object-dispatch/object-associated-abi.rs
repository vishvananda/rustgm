fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{type V;fn f(&self,x:Self::V)->Self::V;}impl A for u8{type V=(u8,u16);fn f(&self,x:Self::V)->Self::V{(*self+x.0,x.1)}}fn main(){let v=7u8;let p:&dyn A<V=(u8,u16)>=&v;let r=p.f((2,1000));check(r.0==9&&r.1==1000);}
