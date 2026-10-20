trait A{type Out;}impl A for u8{type Out=u8;}fn f<T:for<'a>A>(x:T::Out)->T::Out{x}fn main(){if f::<u8>(7)!=7{loop{}}}
