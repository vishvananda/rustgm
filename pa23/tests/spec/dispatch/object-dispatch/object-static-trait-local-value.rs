fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A:'static{fn f(&self)->u8;}impl A for u8{fn f(&self)->u8{*self}}fn read(x:&dyn A)->u8{x.f()}fn main(){let v=7u8;check(read(&v)==7);}
