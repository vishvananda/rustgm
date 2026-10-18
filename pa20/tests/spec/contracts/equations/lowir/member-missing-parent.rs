fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait B{type Out;}trait A<T>{type Out:B<Out=<T as B>::Out>;}fn main(){}