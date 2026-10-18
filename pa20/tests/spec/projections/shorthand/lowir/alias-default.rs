fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{type Out;}type Pair<T:A,U=T::Out>=(T,U);fn main(){}