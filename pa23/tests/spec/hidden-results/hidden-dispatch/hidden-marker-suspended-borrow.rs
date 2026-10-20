fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{fn f(&self)->u8;}impl A for u8{fn f(&self)->u8{*self}}fn main(){let x=make0();check(x.f()==7);}fn need<T:Send>(_:T){}fn make0()->impl A{let n=7u8;let p=&n;need(make1());*p}fn make1()->impl A{let n=9u8;let p=&n;*p}
