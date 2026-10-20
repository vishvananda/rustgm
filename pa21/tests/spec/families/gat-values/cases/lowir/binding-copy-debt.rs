trait F{type Out<U:Copy>;}fn get<T:F<Out<U>=u8>,U>(){}fn main(){}
