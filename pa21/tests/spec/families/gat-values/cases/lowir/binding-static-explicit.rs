trait F{type Out<U:'static>;}fn get<T:F<Out<U>=u8>,U:'static>(){}fn main(){}
