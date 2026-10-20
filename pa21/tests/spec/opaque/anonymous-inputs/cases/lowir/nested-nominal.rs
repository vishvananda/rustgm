fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Value{fn value(&self)->u8;}impl Value for u8{fn value(&self)->u8{*self}}struct W<T>(T);fn f(x:W<impl Value>)->u8{x.0.value()}fn main(){check(f(W(7u8))==7);}
