fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{fn f(&self,x:(u8,u16))->(u16,u8);}impl A for u8{fn f(&self,x:(u8,u16))->(u16,u8){(x.1,*self+x.0)}}fn main(){let v=7u8;let p:&dyn A=&v;let r=p.f((2,1000));check(r.0==1000&&r.1==9);}
