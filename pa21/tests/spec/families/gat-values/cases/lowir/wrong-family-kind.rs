trait F{type Out<'a>;}fn get<T:F<Out<u8>=u8>>(){}fn main(){}
