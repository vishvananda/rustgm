trait F{type Out;fn get()->Self::Out;}struct W<T>(T);impl<T>F for W<T>{type Out=T;fn get()->Self::Out{7u8}}fn main(){}
