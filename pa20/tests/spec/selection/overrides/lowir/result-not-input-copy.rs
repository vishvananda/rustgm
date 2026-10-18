fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{type Out:Copy;}trait Use<T:A>{fn take(x:T)->(T,T);}impl<T:A>Use<T>for u32{fn take(x:T)->(T,T){(x,x)}}fn main(){}