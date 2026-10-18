fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{type Out;}impl A for u8{type Out=u16;}struct Wrap<T:A,U=<T>::Out>(T,U);fn main(){let _:Wrap<u8>=Wrap(0,7);}