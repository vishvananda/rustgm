fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{type Out;}fn f<T:A>(x:<T>::Out)-><T>::Out{x}fn main(){}