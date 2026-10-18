fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{type Out;}impl A for u8{type Out=u16;}type X<T:A>=T::Out;fn keep<T>(x:X<T>)->X<T>{x}fn main(){}