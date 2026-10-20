fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Value{fn value(&self)->u8;}impl Value for u8{fn value(&self)->u8{*self}}struct S;impl S{fn f(&self,x:impl Value)->u8{x.value()}}fn main(){check(S.f(7u8)==7);}
