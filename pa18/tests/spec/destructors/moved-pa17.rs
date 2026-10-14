struct D<T>(T);impl<T> Drop for D<T>{fn drop(&mut self){}}fn main(){let _d=D(7u8);}
