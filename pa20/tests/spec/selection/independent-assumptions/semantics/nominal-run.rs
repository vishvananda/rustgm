fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<U>{}impl B<u8>for u16{}trait C{type Out;}impl<T:B<u8>>C for T{type Out=u16;}struct S<T:Copy,U:B<<T as A>::Out>+B<u8>>where<U as C>::Out:Copy{t:T,u:U}fn main(){let s=S{t:7u8,u:19u16};check(s.t==7);check(s.u==19);}
