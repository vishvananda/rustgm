fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A<'a,T:'a,const N:usize>{type Out;const C:usize=N;}struct S;impl<'a,T:'a,const N:usize>A<'a,T,N>for S{type Out=&'a T;}type X=<S as A<'static,u8,3>>::Out;const C:usize=<S as A<'static,u8,3>>::C;static V:u8=7;fn main(){let x:X=&V;check(*x==7&&C==3);}
