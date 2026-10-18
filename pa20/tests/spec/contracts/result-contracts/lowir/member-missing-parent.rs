fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A<'a>{type Out:'a;}fn wrong<'a,T>(x:<T as A<'a>>::Out){}fn main(){}