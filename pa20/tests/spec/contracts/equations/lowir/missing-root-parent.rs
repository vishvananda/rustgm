fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{type Out:B<Out=u8>;}trait B{type Out;}fn wrong<T>(x:<<T as A>::Out as B>::Out)->u8{x}fn main(){}