fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A:Send{fn f(&self)->u8;}impl A for u8{fn f(&self)->u8{*self}}fn add(x:&dyn A)->&(dyn A+Send){x}fn main(){let x=7u8;check(add(&x).f()==7);}
