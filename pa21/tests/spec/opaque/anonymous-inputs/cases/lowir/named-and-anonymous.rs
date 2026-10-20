fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Value{fn value(&self)->u8;}impl Value for u8{fn value(&self)->u8{*self}}fn f<T:Copy>(t:T,x:impl Value)->u8{let _=t;x.value()}fn main(){check(f::<u16>(1u16,7u8)==7);}
