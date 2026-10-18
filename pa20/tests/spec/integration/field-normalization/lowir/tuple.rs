fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{type Out;}impl A for u8{type Out=u16;}struct W<T:A>(T::Out);fn main(){let w=W::<u8>(7);check(w.0==7);}