fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait F{type Out<const N:usize>;}struct S;impl F for S{type Out<const N:usize>=[u8;N];}fn get<T:F<Out<3>=[u8;3]>>(x:T::Out<3>)->u8{x[1]}fn main(){check(get::<S>([3,7,11])==7);}
