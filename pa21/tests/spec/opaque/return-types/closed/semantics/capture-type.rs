fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Value{fn value(&self)->u8;}impl Value for u8{fn value(&self)->u8{*self}}fn make<T:Value>(x:T)->impl Value{x}fn main(){check(make(7u8).value()==7);}
