fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{type Out;}trait Use<T:A>{fn take(x:T::Out)->T::Out;}impl<T:A>Use<T>for u32{fn take(x:T::Out)->u8{7}}fn main(){}