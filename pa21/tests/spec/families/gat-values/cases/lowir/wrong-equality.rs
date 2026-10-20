trait F{type Out<U>;}struct S;impl F for S{type Out<U>=U;}fn get<T:F<Out<u8>=u16>>(x:T::Out<u8>)->u16{x}fn main(){let _=get::<S>(7);}
