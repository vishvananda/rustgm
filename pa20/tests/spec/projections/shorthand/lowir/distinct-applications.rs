fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A<U>{type Out;}fn f<T:A<u8>+A<u16>>(x:T::Out)->T::Out{x}fn main(){}