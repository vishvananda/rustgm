trait B{type V;}trait F{type Out<U>;}fn get<T:F<Out<<U as B>::V>=u8>,U:B>(){}fn main(){}
