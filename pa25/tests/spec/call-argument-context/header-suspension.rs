fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{const C:usize;}struct X<const N:usize>;trait K<R>{}impl K<fn(u8)->u8> for X<{<S0 as A>::C}>{} const N:u8=accept(id);const fn accept<R:Copy>(_:R)->u8 where X<3>:K<R>{3} fn id(x:u8)->u8{x} struct S0;impl A for S0{const C:usize=3;}fn main(){check(N==3);}
