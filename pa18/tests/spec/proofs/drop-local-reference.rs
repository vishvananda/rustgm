struct D<T>(T);impl<T> Drop for D<T>{fn drop(&mut self){}}fn main(){let d;{let n=7u8;d=D(&n);}}
