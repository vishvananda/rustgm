trait F{type Out<U>;}fn get<T:F<Out<u8>=u8>>(x:T::Out<u16>)->u8{x}fn main(){}
