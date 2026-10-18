fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A<'a> where Self::Out:'a{type Out;}fn wrong<'a,T>(x:<T as A<'a>>::Out){}fn main(){}