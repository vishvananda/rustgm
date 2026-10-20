fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{fn f(&self)->u8;}impl A for u8{fn f(&self)->u8{*self}}fn main(){let v=7u8;let p:&(dyn A+'static)=&v;let f=<dyn A>::f as fn(&(dyn A+'static))->u8;check(f(p)==7);}
