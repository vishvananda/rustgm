trait F{type Out<'a,U>where U:'a;}fn get<'a,T:F<Out<'a,U>=u8>,U:'a>(){}fn main(){}
