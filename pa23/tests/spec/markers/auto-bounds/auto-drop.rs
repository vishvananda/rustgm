fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{fn f(&self)->u8;}impl A for u8{fn f(&self)->u8{*self}}fn erase(x:&(dyn A+Send))->&dyn A{x}fn main(){let x=7u8;let p:&(dyn A+Send)=&x;check(erase(p).f()==7);}
