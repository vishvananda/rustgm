fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A<T>{fn a(&self,x:T)->T;}trait B:A<u16>{}impl A<u16>for u8{fn a(&self,x:u16)->u16{x+2}}impl B for u8{}fn main(){let v=7u8;let p:&dyn B=&v;let q:&dyn A<u16>=p;check(q.a(1000)==1002);}
