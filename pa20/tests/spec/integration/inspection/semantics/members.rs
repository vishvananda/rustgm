fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{type Out:Copy;const C:u8=7;}struct S;struct U;impl A for S{type Out=u8;}impl A for U{type Out=u16;const C:u8=9;}const DEFAULT:u8=<S as A>::C;const OVERRIDE:u8=<U as A>::C;fn f<T:A>(x:T::Out)->T::Out{x}fn main(){check(f::<S>(DEFAULT)==7);check(f::<U>(OVERRIDE as u16)==9);}
