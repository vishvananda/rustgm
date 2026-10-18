fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{type Out;}impl A for u8{type Out=u16;}struct W<T:A>{v:T::Out}fn f<T:A>(x:T::Out)->W<T>{W{v:x}}fn main(){check(f::<u8>(7).v==7);}