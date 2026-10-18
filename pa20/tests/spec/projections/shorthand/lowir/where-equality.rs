fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{type Out;}trait B<U>{}fn f<T:A<Out=T::Out>>(){}fn main(){}