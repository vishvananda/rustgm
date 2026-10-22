fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait V{fn value()->u8;}impl V for u8{fn value()->u8{3}}impl V for u16{fn value()->u8{5}}fn value<T:V>()->u8{T::value()}fn identity<T>(x:T)->T{x}fn main(){check(identity(value::<u8>)()==3);check(identity(value::<u16>)()==5);}
