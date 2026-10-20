fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A<const N:usize>{fn f(&self,x:[u8;N])->u8;}impl A<2>for u8{fn f(&self,x:[u8;2])->u8{x[1]}}fn main(){let v=7u8;let p:&dyn A<2>=&v;check(p.f([2,9])==9);}
